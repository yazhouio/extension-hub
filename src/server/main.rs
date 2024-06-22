use tonic::transport::Server;

use plugin_hub::abi::plugin_hub::plugin_hub_server::PluginHubServer;
use server::MyPluginHub;

extern crate plugin_hub;

mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let greeter = MyPluginHub::default();

    greeter
        .context
        .tar_map
        .insert("aaa".to_owned(), "bbb".to_owned());

    Server::builder()
        .add_service(PluginHubServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
