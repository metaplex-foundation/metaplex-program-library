use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidateError {
    #[error("Missing or empty assets directory")]
    MissingOrEmptyAssetsDirectory,

    #[error("Invalid assets directory")]
    InvalidAssetsDirectory,

    #[error("Name exceeds 32 chars.")]
    NameTooLong,

    #[error("Symbol exceeds 10 chars.")]
    SymbolTooLong,

    #[error("Url exceeds 200 chars.")]
    UrlTooLong,

    #[error("Creator address: {0} is invalid.")]
    InvalidCreatorAddress(String),

    #[error("Creators' share does not equal 100%.")]
    InvalidCreatorShare,

    #[error("Seller fee basis points must be between 0 and 10,000.")]
    InvalidSellerFeeBasisPoints,

    #[error("Missing animation url field")]
    MissingAnimationUrl,

    #[error("Missing external url field")]
    MissingExternalUrl,

    #[error("Missing collection field")]
    MissingCollection,

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
