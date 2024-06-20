use tonic::{transport::Server, Request, Response, Status};

use abi::plugin_hub_server::{PluginHub, PluginHubServer};
pub mod abi {
    tonic::include_proto!("abi");
}

#[derive(Debug, Default)]
pub struct MyPluginHub {}

#[tonic::async_trait]
impl PluginHub for MyPluginHub {
    async fn check_tar(
        &self,
        request: Request<abi::CheckTarRequest>,
    ) -> Result<Response<abi::CheckTarResponse>, Status> {
        println!("Got a request: {:#?}", request);
        let reply = abi::CheckTarResponse {
            error: Some(abi::AppError {
                code: 0,
                message: "A file with the same hash already exists.".into(),
            }),
        };
        Ok(Response::new(reply))
    }

    async fn upload_tar(
        &self,
        request: Request<abi::UploadTarRequest>,
    ) -> Result<Response<abi::UploadTarResponse>, Status> {
        println!("Got a request: {:?}", request);
        let reply = abi::UploadTarResponse {
            upload_url: "Hello, world!".into(),
            error: None,
        };
        Ok(Response::new(reply))
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let greeter = MyPluginHub::default();

    Server::builder()
        .add_service(PluginHubServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
