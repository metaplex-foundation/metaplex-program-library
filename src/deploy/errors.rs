use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeployError {
    #[error("Missing metadata link for cache item {0}")]
    MissingMetadataLink(String),
    #[error("Missing name for cache item {0}")]
    MissingName(String),
}
