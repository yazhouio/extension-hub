use anyhow::Result;
use dashmap::{DashMap, DashSet};
use flate2::read::GzDecoder;
use plugin_hub::error::HubError;
// use plugin_hub::macros::AppError;
use plugin_hub::{abi::plugin_hub as abi, abi::plugin_hub::plugin_hub_server::PluginHub};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::path::PathBuf;
use std::result::Result::Ok;
use std::sync::Arc;
use tar::Archive;
use tokio::time::{sleep, Duration};
use tonic::{Request, Response, Status};

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
    pub tar_map: DashMap<String, String>,
    pub item_dir_map: DashMap<String, DashSet<String>>,
    pub upload_path_map: Arc<DashMap<String, abi::UploadTarRequest>>,
    pub download_path_map: DashMap<String, abi::DownloadTarRequest>,
}

#[derive(Debug, Default)]
pub struct MyPluginHub {
    pub config: MyPluginHubConfig,
    pub context: MyPluginHubContext,
}

impl MyPluginHub {
    pub fn get_tar_hash(&self, tar_hash: &str) -> Result<String, HubError> {
        if self.context.tar_map.contains_key(tar_hash) {
            let relative_path = self
                .context
                .tar_map
                .get(tar_hash)
                .ok_or(HubError::TarNotExist(tar_hash.to_owned()))?
                .to_owned();

            let path = self.config.tar_dir_path.join(&relative_path);
            if path.exists() {
                return Ok(relative_path);
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
        let path = self.config.base_dir.join(item_dir);
        if path.exists() && !overwrite {
            return Err(HubError::DirHasExist(item_dir.to_owned()));
        };
        let file_name = self.get_tar_hash(tar_hash)?;
        let tar_gz = std::fs::File::open(self.config.tar_dir_path.join(&file_name))?;
        // .map_err(|e| HubError::IOError(e))?;

        let tar: GzDecoder<_> = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        if path.exists() && overwrite {
            std::fs::remove_dir_all(&path)?;
        };
        archive.unpack(".")?;
        Ok(())
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
            abi::upload_tar_response::UploadTarData { upload_url: reply },
        )))
    }

    async fn download_tar(
        &self,
        request: Request<abi::DownloadTarRequest>,
    ) -> Result<Response<abi::DownloadTarResponse>, Status> {
        let request = request.into_inner();
        let reply = self.generate_download_url(request)?;
        Ok(abi::DownloadTarResponse::success_response(Some(
            abi::download_tar_response::DownloadTarData {
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
        _request: Request<abi::ReplaceTextRequest>,
    ) -> Result<Response<abi::ReplaceTextResponse>, Status> {
        let reply = abi::ReplaceTextResponse {};
        Ok(Response::new(reply))
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
