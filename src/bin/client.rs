use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::Arc;

use abi::extension_hub_client::ExtensionHubClient;
use anyhow::{anyhow, Result};
use clap::Parser;
use flate2::write::GzEncoder;
use flate2::Compression;
use extension_hub::error::HubErrorCode;
use reqwest::multipart::Part;

pub mod abi {
    tonic::include_proto!("abi");
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Config {
    #[arg(short, long, default_value = "http://127.0.0.1:3000")]
    addr: String,
    #[arg(short, long)]
    extension_name: String,
    #[arg(short, long, value_parser, default_value = "./")]
    dir: PathBuf,
}

impl Config {
    fn dir_to_tar_file(&self) -> Result<(Vec<u8>, String)> {
        let mut output: Vec<u8> = Vec::new();
        {
            let writer = BufWriter::new(&mut output);
            let enc = GzEncoder::new(writer, Compression::default());
            let mut tar = tar::Builder::new(enc);
            tar.append_dir_all("", &self.dir)?;
        }
        let hash = blake3::hash(&output).to_hex();
        Ok((output, hash.to_string()))
    }

    async fn upload_tar(
        &self,
        client: &mut ExtensionHubClient<tonic::transport::Channel>,
        bytes: Arc<Vec<u8>>,
        hash: &str,
    ) -> Result<()> {
        let request = tonic::Request::new(abi::UploadTarRequest {
            tar_hash: hash.to_owned(),
            un_tar: None,
        });

        let file = bytes.as_ref().to_owned();

        let res = client.upload_tar(request).await?.into_inner();
        let upload_url = res.data.ok_or(anyhow!("Upload url not found"))?.upload_url;
        let url = format!("{}/file/{}", &self.addr, upload_url);

        // let file: Vec<u8> = tokio::fs::read("111.tar.gz").await?;
        let part = Part::bytes(file);
        let form = reqwest::multipart::Form::new().part("file", part);

        let response = reqwest::Client::new()
            .post(&url)
            .multipart(form)
            .send()
            .await?;
        if response.status().is_success() {
            println!("Upload tar success: {}", &url);
            Ok(())
        } else {
            anyhow::bail!("Upload tar failed: {} {:?}", &url, response);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Config::parse();
    let addr = if !cli.addr.starts_with("http") {
        "http://".to_owned() + &cli.addr
    } else {
        cli.addr.clone()
    };
    let addr = if cli.addr.ends_with("/") {
        addr[..addr.len() - 1].to_owned()
    } else {
        addr
    };

    println!("Connected to server: {}", addr);
    let mut client: ExtensionHubClient<tonic::transport::Channel> =
        ExtensionHubClient::connect(addr).await?;

    let (file, hash) = cli.dir_to_tar_file()?;
    let file = Arc::new(file);
    let extension_name: Arc<&str> = Arc::new(&cli.extension_name);
    let hash = Arc::new(hash);
    let mut timer = 1;

    loop {
        if timer >= 10 {
            anyhow::bail!("Timeout");
        }
        let request = tonic::Request::new(abi::CheckTarRequest {
            tar_hash: hash.clone().as_ref().to_owned(),
            file_path: extension_name.clone().as_ref().to_owned().to_owned(),
        });
        let response = client.check_tar(request).await;

        match response {
            Ok(_) => {
                println!("Tar already exist");
                break;
            }
            Err(e) => {
                let error: HubErrorCode = HubErrorCode::try_from(e.details()).expect("error code");
                match error {
                    HubErrorCode::TarNotExist => {
                        cli.upload_tar(&mut client, file.clone(), &hash).await?;
                    }
                    HubErrorCode::FileNotExist => {
                        break;
                    }
                    _ => {
                        println!("Error: {:?}", e);
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
        };
        timer += 1;
    }
    let request = tonic::Request::new(abi::UnTarRequest {
        tar_hash: hash.clone().as_ref().to_owned(),
        target_dir: extension_name.clone().as_ref().to_owned().to_owned(),
        overwrite: Some(true),
    });
    let response = client.un_tar(request).await;
    match response {
        Ok(_) => {
            println!("Untar success");
        }
        Err(e) => {
            println!("Untar Error: {:?}", e);
        }
    }
    Ok(())
}
