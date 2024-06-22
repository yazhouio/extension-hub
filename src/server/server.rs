use anyhow::{anyhow, Result};
use dashmap::{DashMap, DashSet};
use plugin_hub::error::HubError;
use plugin_hub::{abi::plugin_hub as abi, abi::plugin_hub::plugin_hub_server::PluginHub};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::path::PathBuf;
use std::result::Result::Ok;
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
    pub upload_path_map: DashMap<String, abi::UploadTarRequest>,
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
            .insert(upload_path.clone(), upload_tar_request)
            .ok_or(anyhow!("Can not insert upload path"))?;
        dbg!(self);
        Ok(upload_path)
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
            Ok(_) => Ok(abi::CheckTarResponse::success_response(None)),
            Err(e) => Err(e.into()),
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
        println!("Got a request: {:?}", request);
        let reply = abi::DownloadTarResponse {
            download_url: "Hello, world!".into(),
        };
        Ok(Response::new(reply))
    }

    async fn un_tar(
        &self,
        request: Request<abi::UnTarRequest>,
    ) -> Result<Response<abi::UnTarResponse>, Status> {
        println!("Got a request: {:?}", request);
        let reply = abi::UnTarResponse { error: None };
        Ok(Response::new(reply))
    }

    async fn replace_text(
        &self,
        request: Request<abi::ReplaceTextRequest>,
    ) -> Result<Response<abi::ReplaceTextResponse>, Status> {
        println!("Got a request: {:?}", request);
        let reply = abi::ReplaceTextResponse { error: None };
        Ok(Response::new(reply))
    }

    async fn clear_tar_dir(
        &self,
        request: Request<abi::ClearTarDirRequest>,
    ) -> Result<Response<abi::ClearTarDirResponse>, Status> {
        println!("Got a request: {:?}", request);
        let reply = abi::ClearTarDirResponse { error: None };
        Ok(Response::new(reply))
    }

    async fn clear_dir(
        &self,
        request: Request<abi::ClearDirRequest>,
    ) -> Result<Response<abi::ClearDirResponse>, Status> {
        println!("Got a request: {:?}", request);
        let reply = abi::ClearDirResponse { error: None };
        Ok(Response::new(reply))
    }
}
