use std::{
    io::{BufRead, Write},
    path::Path,
};

use crate::config::{Config, LocalFs, Server, Ssh, TextReplaceConfigItem};
use anyhow::{Context, Result};
use futures::SinkExt;
use globset::{Glob, GlobSet, GlobSetBuilder};
use opendal::{
    layers,
    services::{Fs, Sftp},
    Operator,
};
use tempfile::NamedTempFile;
use tokio::fs::File;

use tokio_util::io::ReaderStream;
use tracing::debug;

pub struct Helper {
    pub config: Config,
    pub op: Operator,
    context: HelperContext,
}

struct HelperContext {
    pub upload: PathGlobSet,
    pub text: Vec<TextReplaceItem>,
}

pub struct PathGlobSet {
    pub path_ignore_globs: GlobSet,
    pub path_globs: GlobSet,
}

pub struct TextReplaceItem {
    pub path_glob_set: PathGlobSet,
    pub items: Vec<TextReplaceConfigItem>,
}

fn compile_globs(globs: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for glob in globs {
        builder.add(Glob::new(glob).context(format!("Failed to compile glob: {}", glob))?);
    }
    builder.build().context("Failed to compile globs")
}

impl HelperContext {
    pub fn new(config: &Config) -> Result<Self> {
        let upload_config = config
            .upload_config
            .as_ref()
            .context("No upload configuration found.")?;

        let upload = PathGlobSet {
            path_ignore_globs: compile_globs(
                upload_config.path_ignore_globs.as_ref().unwrap_or(&vec![]),
            )?,
            path_globs: compile_globs(upload_config.path_globs.as_ref().unwrap_or(&vec![]))?,
        };

        let text = config
            .text_replace_config
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .map(|item| {
                let path_glob_set = PathGlobSet {
                    path_ignore_globs: compile_globs(&item.path_ignore_globs).unwrap(),
                    path_globs: compile_globs(&item.path_globs).unwrap(),
                };
                TextReplaceItem {
                    path_glob_set,
                    items: item.items.clone(),
                }
            })
            .collect();
        Ok(Self { upload, text })
    }
}

impl Helper {
    pub fn new() -> anyhow::Result<Self> {
        let helper_config = Config::load()?;
        let upload_config = helper_config
            .upload_config
            .as_ref()
            .context("No upload configuration found.")?;

        let context = HelperContext::new(&helper_config)?;
        let server = upload_config
            .server
            .as_ref()
            .context("No server configuration found. Please check your configuration file.")?;
        let op = server
            .to_opendal_operator()
            .context("Failed to create operator. Please check your configuration file.")?;
        let op = op
            .layer(layers::LoggingLayer::default())
            .layer(layers::TimeoutLayer::new())
            .layer(layers::RetryLayer::new().with_max_times(4));
        Ok(Self {
            config: helper_config,
            op,
            context,
        })
    }

    pub async fn text_replace_and_upload<P: AsRef<Path>>(
        &self,
        read: P,
        path: P,
        text_replace_config: &[&TextReplaceItem],
    ) -> Result<()> {
        let mut tmp = NamedTempFile::new()?;
        let mut read = std::fs::File::open(read)?;
        let lines = std::io::BufReader::new(&mut read).lines();
        for line in lines {
            let new_line = text_replace_config.iter().fold(line?, |line, item| {
                item.items
                    .iter()
                    .fold(line, |line, item| line.replace(&item.origin, &item.target))
            });
            tmp.write_all(new_line.as_bytes())?;
            tmp.write_all(b"\n")?;
        }
        tmp.flush()?;

        self.base_upload(tmp.into_temp_path().to_path_buf().as_path(), path.as_ref())
            .await?;
        Ok(())
    }

    pub async fn upload<P: AsRef<Path>>(&self, read: P, path: P) -> Result<()> {
        if self
            .context
            .upload
            .path_ignore_globs
            .is_match(path.as_ref())
        {
            return Ok(());
        }
        if !self.context.upload.path_globs.is_match(path.as_ref()) {
            return Ok(());
        }
        let text_replace_config: Vec<&TextReplaceItem> = self
            .context
            .text
            .iter()
            .filter(|item| {
                item.path_glob_set.path_globs.is_match(path.as_ref())
                    && !item.path_glob_set.path_ignore_globs.is_match(path.as_ref())
            })
            .collect::<Vec<_>>();
        if !text_replace_config.is_empty() {
            self.text_replace_and_upload(read.as_ref(), path.as_ref(), &text_replace_config)
                .await?;
            debug!(
                "Text replace and upload: {:?}",
                path.as_ref().display().to_string()
            );
        } else {
            self.base_upload(read.as_ref(), path.as_ref()).await?;
            debug!("Upload: {:?}", path.as_ref().display().to_string());
        }
        Ok(())
    }

    pub async fn base_upload<P: AsRef<Path>>(&self, origin_path: P, target_path: P) -> Result<()> {
        let reader = File::open(origin_path.as_ref()).await.context(format!(
            "Failed to open file: {:?}",
            origin_path.as_ref().display().to_string()
        ))?;
        let mut w = self
            .op
            .writer_with(&target_path.as_ref().display().to_string())
            .concurrent(8)
            .chunk(256)
            .await?
            .into_bytes_sink();
        let mut file_reader = ReaderStream::new(reader);
        w.send_all(&mut file_reader).await?;
        w.close().await?;
        Ok(())
    }

    pub async fn handler(&self, entry: &walkdir::DirEntry) -> Result<()> {
        if !entry.file_type().is_file() {
            return Ok(());
        }
        let upload_config = self
            .config
            .upload_config
            .as_ref()
            .context("No upload configuration found. Please check your configuration file.")?;
        let file = File::open(entry.path()).await;
        match file {
            Ok(_) => {
                let relative_path = entry
                    .path()
                    .strip_prefix(self.config.origin_dir.as_ref().unwrap_or(&".".to_string()))?;
                // relative_path = relative_path.strip_prefix("/")?;
                let default_path = ".".to_string();
                let path = &upload_config.target_dir.as_ref().unwrap_or(&default_path);
                let path = std::path::Path::new(&path)
                    .to_path_buf()
                    .join(relative_path);
                // let path = path.display().to_string();
                self.upload(entry.path(), path.as_path()).await?;
            }
            Err(e) => {
                eprintln!("Failed to open file: {:?}", e);
            }
        };

        Ok(())
    }
}

impl Ssh {
    // ssh into opendal::Operator
    pub fn to_opendal_operator(&self) -> Result<Operator> {
        let key = std::fs::read_to_string(&self.key_file)
            .context(format!("Failed to read key file: {}", &self.key_file))?;
        let builder = Sftp::default()
            .endpoint(&self.endpoint)
            .user(&self.user)
            .key(&key);
        let op: Operator = Operator::new(builder)?.finish();
        Ok(op)
    }
}

impl LocalFs {
    // ssh into opendal::Operator
    pub fn to_opendal_operator(&self) -> Result<Operator> {
        let builder = Fs::default().root(&self.path);
        let op: Operator = Operator::new(builder)?.finish();
        Ok(op)
    }
}

impl Server {
    pub fn to_opendal_operator(&self) -> Result<Operator> {
        match self {
            Server::Ssh(ssh) => ssh.to_opendal_operator(),
            Server::LocalFs(local_fs) => local_fs.to_opendal_operator(),
        }
    }
}
