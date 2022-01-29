use thiserror::Error;

#[derive(Error, Debug)]
pub enum VerifyError {
    #[error("Failed to get candy machine account data from Solana for address: {0}.")]
    FailedToGetAccountData(String),
}
