use abi::plugin_hub_client::PluginHubClient;

pub mod abi {
    tonic::include_proto!("abi");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PluginHubClient::connect("http://127.0.0.1:50051").await?;

    let request = tonic::Request::new(abi::CheckTarRequest {
        tar_hash: "aaa".into(),
        file_path: "bbb".into(),
    });

    let response = client.check_tar(request).await;

    println!("RESPONSE={:?}", response);

    let request = abi::UploadTarRequest {
        tar_hash: "aaa".into(),
        un_tar: None,
    };
    let response = client.upload_tar(request).await;

    println!("RESPONSE={:?}", response);
    let request = abi::UploadTarRequest {
        tar_hash: "xxxx".into(),
        un_tar: None,
    };
    let response = client.upload_tar(request).await;
    println!("RESPONSE={:?}", response);
    tokio::time::sleep(tokio::time::Duration::from_secs(35)).await;
    let request = abi::UploadTarRequest {
        tar_hash: "yyy".into(),
        un_tar: None,
    };
    let response = client.upload_tar(request).await;
    println!("RESPONSE={:?}", response);
    Ok(())
}
