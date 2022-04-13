use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Could not parse the config file ({0})")]
    ParseError(String),

    #[error("Missing configuration file '{0}'")]
    MissingFileError(String),

    #[error("The configuration file path is invalid ('{0}' is a directory)")]
    InvalidPathError(String),

    #[error("Could not open config file '{0}'")]
    PermissionError(String),

    #[error("Invalid cluster '{0}'")]
    InvalidCluster(String),

    #[error("Invalid upload method '{0}'")]
    InvalidUploadMethod(String),
}
