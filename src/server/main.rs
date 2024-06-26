#![feature(duration_constructors)]

use std::sync::Arc;

use plugin_hub::{abi::plugin_hub::plugin_hub_server::PluginHubServer, error::HubError};
use server::MyPluginHub;

use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
        HeaderMap, StatusCode,
    },
    response::IntoResponse,
    routing::{any_service, get, post},
    Router,
};
use tracing::info;
// use tower_http::limit::RequestBodyLimitLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

extern crate plugin_hub;

mod server;

async fn version() -> &'static str {
    "version 0.1.0"
}

async fn upload(
    State(state): State<Arc<MyPluginHub>>,
    Path(hash): Path<String>,
    mut multipart: Multipart,
) -> Result<(), StatusCode> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let data = field.bytes().await.map_err(|e| {
            tracing::error!("Error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        let result = state.upload_tar(&hash, &data).await.map_err(|e| {
            tracing::error!("Error: {:?}", e);
            println!("Error: {:?}", e);
            match e {
                HubError::ResourceNotFount => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        });
        result?;
    }
    Ok(())
}

async fn download(
    State(state): State<Arc<MyPluginHub>>,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    let (tar_hash, file) = match state.download_tar(&hash) {
        Ok((tar_hash, file)) => (tar_hash, file),
        Err(e) => {
            tracing::error!("Error: {:?}", e);
            return match e {
                HubError::ResourceNotFount => Err(StatusCode::NOT_FOUND),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            };
        }
    };
    let file_name = format!("attachment; filename=\"{}.tar.gz\"", tar_hash);
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/gzip".parse().unwrap());
    headers.insert(CONTENT_DISPOSITION, file_name.parse().unwrap());
    Ok((headers, file))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let addr = "127.0.0.1:3000".parse()?;
    info!("Listening on {}", addr);

    let greeter = MyPluginHub::default();

    // greeter
    //     .context
    //     .tar_map
    //     .insert("aaa".to_owned(), "devops.tar.gz".to_owned());

    let arc_greeter = Arc::new(greeter);

    let grpc_router = Router::new().route(
        "/abi.PluginHub/*rpc",
        any_service(tonic_web::enable(PluginHubServer::from_arc(
            arc_greeter.clone(),
        ))),
    );

    let app = Router::new()
        .route("/file/:hash", get(download))
        .route("/file/:hash", post(upload))
        .layer(DefaultBodyLimit::disable())
        // .layer(RequestBodyLimitLayer::new(
        //     250 * 1024 * 1024, /* 250mb */
        // ))
        // .layer(TraceLayer::new_for_http())
        .with_state(arc_greeter.clone())
        .merge(grpc_router);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
