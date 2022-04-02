use thiserror::Error;

#[derive(Debug, Error)]
pub enum UploadError {
    #[error("Invalid arloader manifest key: {0}")]
    InvalidArloaderManifestKey(String),
    #[error("Missing metadata link for cache item {0}")]
    MissingMetadataLink(String),
}
