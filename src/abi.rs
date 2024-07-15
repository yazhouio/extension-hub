use crate::response_new;
use extension_hub::*;

pub mod extension_hub {
    tonic::include_proto!("abi");
}

response_new!(CheckTarResponse);
response_new!(UploadTarResponse, UploadTarData);
response_new!(DownloadTarResponse, DownloadTarData);
response_new!(ReplaceTextResponse);
response_new!(UnTarResponse);
response_new!(ClearDirResponse);
response_new!(ClearTarDirResponse);

// app_error_to_response!(CheckTarResponse);
// app_error_to_response!(UploadTarResponse, true);
// app_error_to_response!(DownloadTarResponse, true);
// app_error_to_response!(ReplaceTextResponse);
// app_error_to_response!(UnTarResponse);
// app_error_to_response!(ClearDirResponse);
// app_error_to_response!(ClearTarDirResponse);
