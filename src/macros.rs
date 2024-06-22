pub use crate::abi::plugin_hub::*;

#[macro_export]
macro_rules! response_new {
    ($type:ty, $subtype:ty) => {
        impl $type {
            pub fn new(error: Option<AppError>, data: Option<$subtype>) -> Self {
                Self { error, data }
            }

            pub fn success(data: Option<$subtype>) -> Self {
                Self::new(None, data)
            }
            pub fn success_response(data: Option<$subtype>) -> tonic::Response<Self> {
                tonic::Response::new(Self::success(data))
            }
        }
    };
}

#[macro_export]
macro_rules! app_error_to_response {
    ($type:ty) => {
        impl From<AppError> for tonic::Response<$type> {
            fn from(value: AppError) -> Self {
                tonic::Response::new(<$type>::new(Some(value), None))
            }
        }
    };
}
