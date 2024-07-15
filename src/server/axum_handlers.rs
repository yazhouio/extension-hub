use std::sync::Arc;

use crate::file::{path_is_valid, stream_to_file};
use crate::server::MyPluginHub;

use axum::extract::DefaultBodyLimit;
use futures::TryStreamExt;
use plugin_hub::error::HubError;

use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
        HeaderMap, StatusCode,
    },
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use tokio_util::io::ReaderStream;
use tower_http::limit::RequestBodyLimitLayer;

async fn upload(
    State(state): State<Arc<MyPluginHub>>,
    Path(hash): Path<String>,
    mut multipart: Multipart,
) -> Result<(), StatusCode> {
    let config = state
        .context
        .upload_path_map
        .get(&hash)
        .ok_or(StatusCode::NOT_FOUND)?;
    if state.get_tar_hash(&hash).is_ok() {
        if let Some(un_tar) = &config.un_tar {
            return state
                .un_tar_to_dir(
                    &config.tar_hash,
                    &un_tar.target_dir,
                    un_tar.overwrite.unwrap_or(false),
                )
                .await
                .map_err(|e| {
                    tracing::error!("Error: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                });
        } else {
            return Ok(());
        }
    };

    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = format!("{}.tar.gz", config.tar_hash);
        path_is_valid(&file_name).map_err(|e| {
            tracing::error!("Error: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;
        let target_path = state.config.tar_dir_path.join(&file_name);
        let mut tmp_file = state.config.tar_dir_path.join("__tmp__");
        tmp_file.push(&file_name);
        let path = stream_to_file(
            tmp_file,
            field.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err)),
        )
        .await
        .map_err(|e| {
            tracing::error!("Error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let result = state
            .upload_tar_by_path(&hash, &path, &target_path)
            .await
            .map_err(|e| {
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
    let (tar_hash, file) = match state.get_download_tar_path(&hash) {
        Ok((tar_hash, file)) => (tar_hash, file),
        Err(e) => {
            tracing::error!("Error: {:?}", e);
            return match e {
                HubError::ResourceNotFount => Err((StatusCode::NOT_FOUND, e.to_string())),
                _ => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {:?}", e))),
            };
        }
    };
    let file_name = format!("attachment; filename=\"{}.tar.gz\"", tar_hash);
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/gzip".parse().unwrap());
    headers.insert(CONTENT_DISPOSITION, file_name.parse().unwrap());
    // convert the `AsyncRead` into a `Stream`
    let file = match tokio::fs::File::open(file).await {
        Ok(file) => file,
        Err(e) => {
            tracing::error!("Error: {:?}", e);
            return Err((StatusCode::NOT_FOUND, format!("File not found: {}", e)));
        }
    };

    let stream = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = Body::from_stream(stream);
    Ok((headers, body).into_response())
}

pub fn router(state: Arc<MyPluginHub>) -> Router {
    Router::new()
        .route("/version", get(|| async { "0.1.0" }))
        .route("/file/:hash", get(download))
        .route("/file/:hash", post(upload))
        .with_state(state.clone())
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            250 * 1024 * 1024, /* 250mb */
        ))
}
