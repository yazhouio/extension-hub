use anyhow::{Context, Result};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use walkdir::WalkDir;

#[allow(dead_code)]
mod config;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");
    info!("Starting up");
    let helper = utils::Helper::new()?;
    helper.config.validate()?;
    let dir = helper.config.origin_dir.as_deref().unwrap_or(".");
    for entry in WalkDir::new(dir) {
        let entry = entry.context(format!("Failed to read entry: {:?}", dir))?;
        helper.handler(&entry).await?;
    }
    info!("Finished");
    Ok(())
}
