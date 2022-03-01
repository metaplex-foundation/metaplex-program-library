use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Error parsing the config file: {0}")]
    ParseError(String),

    #[error("Error opening the config file: {0}")]
    FileOpenError(String),
    #[error("Invalid cluster: {0}")]
    InvalidCluster(String),
}
