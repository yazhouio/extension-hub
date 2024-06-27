use anyhow::Result;
use dashmap::{DashMap, DashSet};
use flate2::read::GzDecoder;
use plugin_hub::error::HubError;
use plugin_hub::text_replace;
// use plugin_hub::macros::AppError;
use plugin_hub::{abi::plugin_hub as abi, abi::plugin_hub::plugin_hub_server::PluginHub};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::result::Result::Ok;
use std::sync::Arc;
use tar::Archive;
use tokio::time::{sleep, Duration};
use tonic::{Request, Response, Status};
use tracing::debug;

use crate::file::path_is_valid;

extern crate plugin_hub;

#[derive(Debug)]
pub struct MyPluginHubConfig {
    pub base_dir: PathBuf,
    pub tar_dir_path: PathBuf,
}

impl Default for MyPluginHubConfig {
    fn default() -> Self {
        let path = PathBuf::from("/tmp/plugin_hub");
        MyPluginHubConfig {
            base_dir: path.clone(),
            tar_dir_path: path.join("__tar"),
        }
    }
}

#[derive(Debug, Default)]
pub struct MyPluginHubContext {
    pub tar_set: DashSet<String>,
    pub item_dir_map: DashMap<String, DashSet<String>>,
    pub upload_path_map: Arc<DashMap<String, abi::UploadTarRequest>>,
    pub download_path_map: Arc<DashMap<String, abi::DownloadTarRequest>>,
}

#[derive(Debug, Default)]
pub struct MyPluginHub {
    pub config: MyPluginHubConfig,
    pub context: MyPluginHubContext,
}

impl MyPluginHub {
    pub fn get_tar_hash(&self, tar_hash: &str) -> Result<String, HubError> {
        if self.context.tar_set.contains(tar_hash) {
            let tar_file = format!("{}.tar.gz", tar_hash);
            path_is_valid(&tar_file)?;
            let path = self.config.tar_dir_path.join(&tar_file);
            if path.exists() {
                return Ok(tar_hash.to_owned());
            };
        }
        Err(HubError::TarNotExist(tar_hash.to_owned()))
    }

    pub fn check_tar_dir(&self, tar_hash: &str, item_dir: &str) -> Result<(), HubError> {
        let file_name = self.get_tar_hash(tar_hash)?;
        if self.context.item_dir_map.contains_key(&file_name) {
            let set = self
                .context
                .item_dir_map
                .get(&file_name)
                .ok_or(HubError::DirNotExist(file_name.to_owned()))?;
            if set.contains(item_dir) {
                let path = self.config.base_dir.join(item_dir);
                path_is_valid(&item_dir)?;
                if path.exists() && path.is_dir() {
                    return Ok(());
                }
            }
        }
        Err(HubError::FileNotExist(item_dir.to_owned()))
    }

    pub fn generate_upload_url(
        &self,
        upload_tar_request: abi::UploadTarRequest,
    ) -> Result<String, HubError> {
        let upload_path: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        self.context
            .upload_path_map
            .insert(upload_path.clone(), upload_tar_request);
        let path_clone = upload_path.clone();
        let upload_path_map = self.context.upload_path_map.clone();
        tokio::task::spawn(async move {
            let sleep_time = Duration::from_secs(30);
            sleep(sleep_time).await;
            upload_path_map.remove(&path_clone);
        });
        Ok(upload_path)
    }

    pub fn generate_download_url(
        &self,
        download_tar_request: abi::DownloadTarRequest,
    ) -> Result<String, HubError> {
        let download_path: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        self.context
            .download_path_map
            .insert(download_path.clone(), download_tar_request);
        let path_clone = download_path.clone();
        let download_path_map = self.context.download_path_map.clone();
        tokio::task::spawn(async move {
            let sleep_time = Duration::from_mins(30);
            sleep(sleep_time).await;
            download_path_map.remove(&path_clone);
        });
        Ok(download_path)
    }

    pub async fn un_tar_to_dir(
        &self,
        tar_hash: &str,
        item_dir: &str,
        overwrite: bool,
    ) -> Result<(), HubError> {
        path_is_valid(item_dir)?;
        let path = self.config.base_dir.join(item_dir);
        if path.exists() && !overwrite {
            return Err(HubError::DirHasExist(item_dir.to_owned()));
        };
        let mut file_name = self.get_tar_hash(tar_hash)?;
        file_name.push_str(".tar.gz");
        path_is_valid(&file_name)?;
        let tar_gz = std::fs::File::open(self.config.tar_dir_path.join(&file_name))?;

        let tar: GzDecoder<_> = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        if path.exists() && overwrite {
            std::fs::remove_dir_all(&path)?;
        };
        archive.unpack(path)?;
        self.add_tar_dir(tar_hash, item_dir);
        Ok(())
    }

    pub fn text_replace_request_to_setting(
        &self,
        request: abi::ReplaceTextRequest,
    ) -> Result<text_replace::Setting, HubError> {
        let abi::ReplaceTextRequest {
            target_dir,
            old_text,
            new_text,
            suffix,
        } = request;

        let source_path = self.config.base_dir.join(target_dir);
        let output_path = source_path.clone();
        Ok(text_replace::Setting {
            old_web_prefix: old_text,
            new_web_prefix: new_text,
            source_path: source_path.to_string_lossy().to_string(),
            output_path: output_path.to_string_lossy().to_string(),
            exclude_path: None,
            file_types: Some(suffix),
        })
    }

    pub fn text_replace_by_request(
        &self,
        request: abi::ReplaceTextRequest,
    ) -> Result<(), HubError> {
        let config = self.text_replace_request_to_setting(request)?;
        Ok(config.text_replace()?)
    }

    #[warn(clippy::unwrap_or_default)]
    pub fn add_tar_dir(&self, tar_hash: &str, item_dir: &str) {
        let set = self
            .context
            .item_dir_map
            .entry(tar_hash.to_owned())
            .or_insert(DashSet::new());
        set.insert(item_dir.to_owned());
    }
    pub async fn upload_tar(&self, hash: &str, bytes: &[u8]) -> Result<(), HubError> {
        let Some(_request) = self.context.upload_path_map.get(hash) else {
            return Err(HubError::ResourceNotFount);
        };
        if !self.config.tar_dir_path.exists() {
            std::fs::create_dir_all(&self.config.tar_dir_path)?;
        };
        let request = _request.clone();
        let file_name = format!("{}.tar.gz", request.tar_hash);
        let path = self.config.tar_dir_path.join(&file_name);
        if !path.exists() {
            let request = _request.clone();
            let hasher = blake3::hash(bytes);
            let hash_str = hasher.to_hex().to_string();
            if hash_str != request.clone().tar_hash {
                return Err(HubError::HashNotMatch(request.clone().tar_hash, hash_str));
            };
            let mut file = std::fs::File::create(path)?;
            file.write_all(bytes)?;
        };
        let request = _request.clone();
        self.context
            .tar_set
            .insert(_request.clone().tar_hash.to_owned());
        let Some(un_tar_request) = request.un_tar else {
            return Ok(());
        };
        let request = _request.clone();
        self.un_tar_to_dir(
            &request.tar_hash.clone(),
            &un_tar_request.target_dir,
            un_tar_request.overwrite.unwrap_or(false),
        )
        .await
    }

    pub async fn upload_tar_by_path(
        &self,
        hash: &str,
        path: impl AsRef<Path>,
        target_path: impl AsRef<Path>,
    ) -> Result<(), HubError> {
        let Some(_request) = self.context.upload_path_map.get(hash) else {
            return Err(HubError::ResourceNotFount);
        };
        let target_path = target_path.as_ref();
        let path = path.as_ref();
        let bytes = tokio::fs::read(&path).await?;
        let request = _request.clone();
        let hasher = blake3::hash(&bytes);
        let hash_str = hasher.to_hex().to_string();
        if hash_str != request.clone().tar_hash {
            return Err(HubError::HashNotMatch(request.clone().tar_hash, hash_str));
        };
        std::fs::rename(&path, target_path).map_err(|e| {
            debug!(
                "Got error: {:?}, when move file {:?} to {:?}",
                e, &path, &target_path
            );
            HubError::IOError(e)
        })?;
        self.upload_tar(hash, &bytes).await?;
        Ok(())
    }

    pub fn download_tar(&self, url: &str) -> Result<(String, Vec<u8>), HubError> {
        let Some(_request) = self.context.download_path_map.get(url) else {
            return Err(HubError::ResourceNotFount);
        };
        let request = _request.clone();
        let tar_file_name = format!("{}.tar.gz", request.clone().tar_hash);
        path_is_valid(&tar_file_name)?;
        let path = self.config.tar_dir_path.join(tar_file_name);
        if !path.exists() {
            return Err(HubError::TarNotExist(request.clone().tar_hash));
        };
        let mut file = std::fs::File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok((request.clone().tar_hash, bytes))
    }

    pub fn get_download_tar_path(&self, url: &str) -> Result<(String, String), HubError> {
        let Some(_request) = self.context.download_path_map.get(url) else {
            return Err(HubError::ResourceNotFount);
        };
        let request = _request.clone();
        let tar_file_name = format!("{}.tar.gz", request.clone().tar_hash);
        path_is_valid(&tar_file_name)?;
        let path = self.config.tar_dir_path.join(tar_file_name);
        if !path.exists() {
            return Err(HubError::TarNotExist(request.clone().tar_hash));
        };
        Ok((request.clone().tar_hash, path.to_string_lossy().to_string()))
    }
}

#[tonic::async_trait]
impl PluginHub for MyPluginHub {
    async fn check_tar(
        &self,
        request: Request<abi::CheckTarRequest>,
    ) -> Result<Response<abi::CheckTarResponse>, Status> {
        let abi::CheckTarRequest {
            tar_hash,
            file_path,
        } = request.into_inner();

        match self.check_tar_dir(&tar_hash, &file_path) {
            Ok(_) => Ok(abi::CheckTarResponse::success_response()),
            Err(e) => {
                let status = e.into();
                Err(status)
            }
        }
    }

    async fn upload_tar(
        &self,
        request: Request<abi::UploadTarRequest>,
    ) -> Result<Response<abi::UploadTarResponse>, Status> {
        let request = request.into_inner();
        let reply = self.generate_upload_url(request)?;
        Ok(abi::UploadTarResponse::success_response(Some(
            abi::UploadTarData { upload_url: reply },
        )))
    }

    async fn download_tar(
        &self,
        request: Request<abi::DownloadTarRequest>,
    ) -> Result<Response<abi::DownloadTarResponse>, Status> {
        let request = request.into_inner();
        let reply = self.generate_download_url(request)?;
        Ok(abi::DownloadTarResponse::success_response(Some(
            abi::DownloadTarData {
                download_url: reply,
            },
        )))
    }

    async fn un_tar(
        &self,
        request: Request<abi::UnTarRequest>,
    ) -> Result<Response<abi::UnTarResponse>, Status> {
        let abi::UnTarRequest {
            tar_hash,
            target_dir,
            overwrite,
        } = request.into_inner();
        let reply = self
            .un_tar_to_dir(&tar_hash, &target_dir, overwrite.unwrap_or(false))
            .await;
        match reply {
            Ok(_) => Ok(abi::UnTarResponse::success_response()),
            Err(e) => Err(e.into()),
        }
    }

    async fn replace_text(
        &self,
        request: Request<abi::ReplaceTextRequest>,
    ) -> Result<Response<abi::ReplaceTextResponse>, Status> {
        let request = request.into_inner();
        let reply = self.text_replace_by_request(request);
        match reply {
            Ok(_) => Ok(abi::ReplaceTextResponse::success_response()),
            Err(e) => Err(e.into()),
        }
    }

    async fn clear_tar_dir(
        &self,
        _request: Request<abi::ClearTarDirRequest>,
    ) -> Result<Response<abi::ClearTarDirResponse>, Status> {
        let reply = abi::ClearTarDirResponse {};
        Ok(Response::new(reply))
    }

    async fn clear_dir(
        &self,
        _request: Request<abi::ClearDirRequest>,
    ) -> Result<Response<abi::ClearDirResponse>, Status> {
        let reply = abi::ClearDirResponse {};
        Ok(Response::new(reply))
    }
}
