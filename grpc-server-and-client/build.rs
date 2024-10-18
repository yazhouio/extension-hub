fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        // .type_attribute("UploadTarData", "#[derive(Debug)]")
        // .type_attribute("UploadTarRequest", "#[derive(Debug)]")
        // .type_attribute("DownloadTarRequest", "#[derive(Debug)]")
        // .type_attribute("UnTarRequest", "#[derive(Debug)]")
        // .type_attribute("ReplaceTextRequest", "#[derive(Debug)]")
        .compile(&["proto/api.proto"], &["proto"])?;
    Ok(())
}
