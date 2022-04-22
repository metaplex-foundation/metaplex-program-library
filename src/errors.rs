use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SetupError {
    #[error("Error setting up sugar: {0}")]
    SugarSetupError(String),
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Cache file '{0}' not found. Run `sugar upload` to create it or provide it with the --cache option.")]
    CacheFileNotFound(String),

    #[error("Invalid candy machine address: {0}")]
    InvalidCandyMachineAddress(String),

    #[error("Failed to open cache file: {0} with error: {1}")]
    FailedToOpenCacheFile(String, String),

    #[error("Failed to parse cache file with error: {0}")]
    CacheFileWrongFormat(String),
}

#[derive(Debug, Error)]
pub enum ReadFilesError {
    #[error("Path errors, check log file for details.")]
    PathErrors,

    #[error("Deserialize errors, check log file for details.")]
    DeserializeErrors,

    #[error("Validate errors, check log file for details.")]
    ValidateErrors,

    #[error("File open errors, check log file for details.")]
    FileOpenErrors,
}

#[derive(Debug)]
pub struct DeserializeError<'a> {
    pub path: &'a PathBuf,
    pub error: serde_json::Error,
}

#[derive(Debug)]
pub struct FileOpenError<'a> {
    pub path: &'a PathBuf,
    pub error: std::io::Error,
}
