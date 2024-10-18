pub use crate::abi::extension_hub::*;

#[macro_export]
macro_rules! response_new {
    ($type:ty, $subtype:ty) => {
        impl $type {
            pub fn new(data: Option<$subtype>) -> Self {
                Self { data }
            }

            pub fn success(data: Option<$subtype>) -> Self {
                Self::new(data)
            }
            pub fn success_response(data: Option<$subtype>) -> tonic::Response<Self> {
                tonic::Response::new(Self::success(data))
            }
        }
    };
    ($type:ty) => {
        impl $type {
            pub fn new() -> Self {
                Self {}
            }

            pub fn success() -> Self {
                Self::new()
            }
            pub fn success_response() -> tonic::Response<Self> {
                tonic::Response::new(Self::success())
            }
        }
    };
}

// #[macro_export]
// macro_rules! app_error_to_response {
//     ($type:ty, $_: expr) => {
//         impl From<AppError> for tonic::Response<$type> {
//             fn from(value: AppError) -> Self {
//                 tonic::Response::new(<$type>::new(Some(value), None))
//             }
//         }
//     };
//     ($type:ty) => {
//         impl From<AppError> for tonic::Response<$type> {
//             fn from(value: AppError) -> Self {
//                 tonic::Response::new(<$type>::new(Some(value)))
//             }
//         }
//     };
// }
