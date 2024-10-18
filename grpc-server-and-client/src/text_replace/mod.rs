use std::{fs, path::PathBuf};

use anyhow::{bail, Result};
use globset::Glob;

#[derive(Debug)]
pub struct Setting {
    pub old_web_prefix: String,
    pub new_web_prefix: String,
    pub exclude_path: Option<Vec<String>>,
    pub source_path: String,
    pub output_path: String,
    pub file_types: Option<Vec<String>>,
}

impl Setting {
    pub fn text_replace(&self) -> Result<()> {
        map_files(self)
    }
}

pub fn map_files(setting: &Setting) -> Result<()> {
    let Setting {
        source_path,
        file_types,
        exclude_path,
        old_web_prefix,
        new_web_prefix,
        output_path,
    } = setting;
    let file_types: Vec<String> = file_types.to_owned().unwrap_or(vec![]);
    let mut exclude_path = exclude_path.to_owned().unwrap_or(vec![]);
    exclude_path.push("\\.git$".to_owned());
    let path = PathBuf::from(&source_path);
    if !path.exists() {
        bail!("dir not exists");
    }
    let walker = walkdir::WalkDir::new(source_path)
        .into_iter()
        .filter_entry(|e| {
            let relative_path = e.path().strip_prefix(source_path).unwrap();
            !exclude_path.iter().any(|p| {
                let glob = Glob::new(p)
                    .unwrap_or_else(|_| panic!("invalid glob pattern: {}", p))
                    .compile_matcher();
                glob.is_match(relative_path.to_str().unwrap_or_default())
            })
        });
    for entry in walker {
        let entry = entry?;
        let path = entry.path();
        let is_file = path.is_file();
        if !is_file {
            continue;
        }
        let file_type = path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string();
        if !file_types.contains(&file_type) {
            continue;
        }
        let content = fs::read_to_string(path)?;
        if !content.contains(old_web_prefix) {
            continue;
        }
        let new_content = content.replace(old_web_prefix, new_web_prefix);
        let path = path.strip_prefix(source_path)?;
        let out_dir = PathBuf::from(&output_path);
        let path = out_dir.join(path);
        let parent = path.parent().unwrap();
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, new_content)?;
    }
    Ok(())
}
