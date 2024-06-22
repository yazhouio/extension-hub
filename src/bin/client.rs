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

    let response = client.check_tar(request).await?;

    println!("RESPONSE={:?}", response);

    // TODO
    Ok(())
}
