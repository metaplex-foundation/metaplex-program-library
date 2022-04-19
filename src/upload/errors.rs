use thiserror::Error;

#[derive(Debug, Error)]
pub enum UploadError {
    #[error("Invalid assets directory: {0}")]
    InvalidAssetsDirectory(String),

    #[error("Failed to get extension from assets dir")]
    GetExtensionError,

    #[error("No extension for path")]
    NoExtension,

    #[error("Invalid number of files {0}, there should be an even number of files")]
    InvalidNumberOfFiles(usize),

    #[error("No Bundlr balance found for address: {0}, check Bundlr cluster and address balance")]
    NoBundlrBalance(String),

    #[error("Invalid Bundlr cluster: {0} Use 'devnet' or 'mainnet'")]
    InvalidBundlrCluster(String),

    #[error("{0}")]
    Incomplete(String),

    #[error("{0}")]
    SendDataFailed(String),

    #[error(
        "Mismatch value for \"{0}\" property in file \"{1}\": expected \"{2}\", found \"{3}\""
    )]
    MismatchValue(String, String, String, String),
}
