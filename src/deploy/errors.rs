use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeployError {
    #[error("Missing metadata link for cache item {0}")]
    MissingMetadataLink(String),
    #[error("Missing name for cache item {0}")]
    MissingName(String),
    #[error("{0}")]
    AddConfigLineFailed(String),
    #[error(
        "Your current wallet balance of {0} SOL is not enough. {1} SOL is needed to deploy the candy machine."
    )]
    BalanceTooLow(String, String),
}
