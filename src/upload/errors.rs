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

    #[error("{0}")]
    Incomplete(String),

    #[error("{0}")]
    SendDataFailed(String),

    #[error(
        "Mismatch value for \"{0}\" property in file \"{1}\": expected \"{2}\", found \"{3}\""
    )]
    MismatchValue(String, String, String, String),

    #[error("Metadata file {0} is not formatted correctly for animations.")]
    AnimationFileError(String),
}
