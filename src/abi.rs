use crate::{app_error_to_response, response_new};
use plugin_hub::*;

pub mod plugin_hub {
    tonic::include_proto!("abi");
}

response_new!(CheckTarResponse, check_tar_response::CheckTarData);
response_new!(UploadTarResponse, upload_tar_response::UploadTarData);

app_error_to_response!(CheckTarResponse);
app_error_to_response!(UploadTarResponse);
