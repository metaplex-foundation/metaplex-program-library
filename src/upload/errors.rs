use thiserror;

pub enum UploadErrors {
    #[error("Invalid arloader manifest key: {0}")]
    InvalidArloaderManifestKey(String),
}
