use abi::plugin_hub_client::PluginHubClient;

pub mod abi {
    tonic::include_proto!("abi");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PluginHubClient::connect("http://127.0.0.1:3000").await?;

    // let request = tonic::Request::new(abi::CheckTarRequest {
    //     tar_hash: "aaa".into(),
    //     file_path: "bbb".into(),
    // });

    // let response = client.check_tar(request).await;
    // println!("RESPONSE={:?}", response);

    let request = tonic::Request::new(abi::UploadTarRequest {
        tar_hash: "f82d839cf18aed8d79131c0bec2afad50164f3806d8d5c60a054a720916830e2".into(),
        un_tar: Some(abi::UnTarRequest {
            tar_hash: "f82d839cf18aed8d79131c0bec2afad50164f3806d8d5c60a054a720916830e2".into(),
            target_dir: "./devops".into(),
            overwrite: Some(true),
        }),
    });
    let response = client.upload_tar(request).await;
    println!("RESPONSE={:?}", response);

    let request = tonic::Request::new(abi::DownloadTarRequest {
        tar_hash: "f82d839cf18aed8d79131c0bec2afad50164f3806d8d5c60a054a720916830e2".into(),
    });
    let response = client.download_tar(request).await;
    println!("RESPONSE={:?}", response);

    // let request = tonic::Request::new(abi::UnTarRequest {
    //     tar_hash: "aaa".into(),
    //     target_dir: "./devops".into(),
    //     overwrite: Some(true),
    // });
    // let response = client.un_tar(request).await;
    // println!("RESPONSE={:?}", response);

    // let request = tonic::Request::new(abi::ReplaceTextRequest {
    //     target_dir: "devops".to_owned(),
    //     old_text: "log".to_owned(),
    //     new_text: "ccc".to_owned(),
    //     suffix: vec!["js".to_owned()],
    // });
    // let response = client.replace_text(request).await;
    // println!("RESPONSE={:?}", response);
    Ok(())
}
