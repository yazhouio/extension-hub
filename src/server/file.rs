use bytes::Bytes;
use futures::Stream;
use std::path::{Path, PathBuf};
use tokio::{fs, fs::File, io::BufWriter};
use tokio_util::io::StreamReader;

use plugin_hub::error::HubError;

extern crate plugin_hub;

pub fn path_is_valid(path: impl AsRef<Path>) -> Result<(), HubError> {
    let path = path.as_ref();
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return Err(HubError::InvalidPath(path.to_string_lossy().into_owned()));
        }
    }

    if components.count() != 1 {
        return Err(HubError::InvalidPath(path.to_string_lossy().into_owned()));
    }

    Ok(())
}

pub async fn stream_to_file<S>(path: impl AsRef<Path>, stream: S) -> Result<PathBuf, HubError>
where
    S: Stream<Item = Result<Bytes, std::io::Error>>,
{
    let path = path.as_ref();
    if path.parent().is_none() {
        return Err(HubError::InvalidPath(path.to_string_lossy().into_owned()));
    }

    if let Some(parent) = path.parent() {
        dbg!(parent);
        if !parent.exists() {
            fs::create_dir_all(parent)
                .await
                .map_err(HubError::IOError)?;
        }
    }

    // Convert the stream into an `AsyncRead`.
    let body_with_io_error = stream;
    let body_reader = StreamReader::new(body_with_io_error);
    futures::pin_mut!(body_reader);

    // Create the file. `File` implements `AsyncWrite`.
    let mut file = BufWriter::new(File::create(&path).await?);

    // Copy the body into the file.
    tokio::io::copy(&mut body_reader, &mut file).await?;

    Ok(path.into())
}

// pub async fn file_to_stream(
//     path: impl AsRef<Path>,
//     root: impl AsRef<Path>,
// ) -> Result<impl Stream<Item = Result<Bytes, std::io::Error>>, HubError> {
//     path_is_valid(&path)?;

//     let path = root.as_ref().join(path);
//     let file = std::fs::File::open(path)?;

//     Ok(file.into_async_read())
// }
