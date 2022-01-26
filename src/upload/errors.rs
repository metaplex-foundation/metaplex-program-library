use thiserror::Error;

#[derive(Debug, Error)]
pub enum UploadErrors {
    #[error("Invalid arloader manifest key: {0}")]
    InvalidArloaderManifestKey(String),
}
