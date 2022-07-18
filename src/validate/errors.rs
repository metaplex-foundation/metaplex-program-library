use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum ValidateParserError {
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

    #[error("Creator address: '{0}' is invalid.")]
    InvalidCreatorAddress(String),

    #[error("Combined creators' share does not equal 100%.")]
    InvalidCreatorShare,

    #[error("Seller fee basis points value '{0}' is invalid: must be between 0 and 10,000.")]
    InvalidSellerFeeBasisPoints(u16),

    #[error("Missing animation url field")]
    MissingAnimationUrl,

    #[error("Missing external url field")]
    MissingExternalUrl,

    #[error("Missing collection field")]
    MissingCollection,

    #[error("Missing creators field")]
    MissingCreators,

    #[error("Missing seller fee basis points field")]
    MissingSellerFeeBasisPoints,
}
