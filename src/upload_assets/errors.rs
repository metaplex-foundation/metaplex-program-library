use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UploadAssetsError {
    #[error("Invalid assets directory: {0}")]
    InvalidAssetsDirectory(String),

    #[error("Failed to get extension from assets dir")]
    GetExtensionError,

    #[error("No extension for path")]
    NoExtension,

    #[error("Invalid number of files: {0}. There should be an even number of files.")]
    InvalidNumberOfFiles(usize),

    #[error("No Bundlr balance found for address: {0}")]
    NoBundlrBalance(String),
}

pub struct DeserializeError<'a> {
    pub path: &'a PathBuf,
    pub error: serde_json::Error,
}

pub struct FileOpenError<'a> {
    pub path: &'a PathBuf,
    pub error: std::io::Error,
}
