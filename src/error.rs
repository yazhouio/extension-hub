use thiserror::Error;
use tonic::Status;

use crate::abi::plugin_hub as abi;

#[derive(Error, Debug)]
pub enum HubError {
    #[error("Tar package with hash '{0}' not exist")]
    TarNotExist(String),

    #[error("File '{0}' not exist")]
    FileNotExist(String),

    #[error("Directory '{0}' not exist")]
    DirNotExist(String),

    #[error("Can not found the config")]
    ConfigNotExist,

    #[error("Configure error: {0}")]
    ConfigureError(String),

    #[error("IO error: {0}")]
    IOError(String),

    #[error(transparent)]
    OtherError(#[from] anyhow::Error),

    // detailed errors
    #[error("Unsupported API: {0}")]
    UnsupportedApi(String),
    #[error("Malformed API response for {0}")]
    MalformedApiResponse(String),

    // converted errors
    #[error("Protobuf decode error: {0}")]
    ProstDecodeError(#[from] prost::DecodeError),
    #[error("Protobuf decode error: {0}")]
    ProstEncodeError(#[from] prost::EncodeError),
}

impl From<HubError> for Status {
    fn from(err: HubError) -> Self {
        match err {
            HubError::TarNotExist(hash) => {
                Status::not_found(format!("Tar package with hash '{}' not exist", hash))
            }
            HubError::FileNotExist(file) => Status::not_found(format!("File '{}' not exist", file)),
            HubError::DirNotExist(dir) => {
                Status::not_found(format!("Directory '{}' not exist", dir))
            }
            HubError::ConfigNotExist => Status::not_found("Can not found the config"),
            HubError::ConfigureError(msg) => {
                Status::invalid_argument(format!("Configure error: {}", msg))
            }
            HubError::IOError(msg) => Status::internal(format!("IO error: {}", msg)),
            HubError::UnsupportedApi(api) => {
                Status::unimplemented(format!("Unsupported API: {}", api))
            }
            HubError::MalformedApiResponse(api) => {
                Status::internal(format!("Malformed API response for {}", api))
            }
            HubError::ProstDecodeError(err) => {
                Status::internal(format!("Protobuf decode error: {}", err))
            }
            HubError::ProstEncodeError(err) => {
                Status::internal(format!("Protobuf encode error: {}", err))
            },
            HubError::OtherError(err) => Status::internal(format!("Other error: {}", err)),
        }
    }
}

impl From<HubError> for abi::AppError {
    fn from(value: HubError) -> Self {
        match value {
            HubError::ConfigNotExist => abi::AppError {
                code: abi::AppErrorCode::ConfigNotExist.into(),
                message: Some("Can not found the config".to_string()),
            },
            HubError::TarNotExist(hash) => abi::AppError {
                code: abi::AppErrorCode::TarNotExist.into(),
                message: Some(format!("Tar package with hash '{}' not exist", hash)),
            },
            HubError::FileNotExist(file) => abi::AppError {
                code: abi::AppErrorCode::FileNotExist.into(),
                message: Some(format!("File '{}' not exist", file)),
            },
            HubError::DirNotExist(dir) => abi::AppError {
                code: abi::AppErrorCode::DirNotExist.into(),
                message: Some(format!("Directory '{}' not exist", dir)),
            },
            HubError::ConfigureError(msg) => abi::AppError {
                code: abi::AppErrorCode::ConfigureError.into(),
                message: Some(format!("Configure error: {}", msg)),
            },
            HubError::IOError(msg) => abi::AppError {
                code: abi::AppErrorCode::IoError.into(),
                message: Some(format!("IO error: {}", msg)),
            },
            HubError::UnsupportedApi(api) => abi::AppError {
                code: abi::AppErrorCode::UnsupportedApi.into(),
                message: Some(format!("Unsupported API: {}", api)),
            },
            HubError::MalformedApiResponse(api) => abi::AppError {
                code: abi::AppErrorCode::MalformedApiResponse.into(),
                message: Some(format!("Malformed API response for {}", api)),
            },
            HubError::ProstDecodeError(err) => abi::AppError {
                code: abi::AppErrorCode::ProstDecodeError.into(),
                message: Some(format!("Protobuf decode error: {}", err)),
            },
            HubError::ProstEncodeError(err) => abi::AppError {
                code: abi::AppErrorCode::ProstEncodeError.into(),
                message: Some(format!("Protobuf encode error: {}", err)),
            },
            HubError::OtherError(err) => abi::AppError {
                code: abi::AppErrorCode::Other.into(),
                message: Some(format!("Other error: {}", err)),
            },
        }
    }
}
