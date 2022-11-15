use thiserror::Error;

#[derive(Debug, Error)]
pub enum AirDropError {
    #[error("AirDrop list file {0} not found")]
    AirDropListFileNotFound(String),

    #[error("Failed to open AirDrop list file {0} with error {1}")]
    FailedToOpenAirDropListFile(String, String),

    #[error("Failed to parse AirDrop list file {0} with error {1}")]
    AirDropListFileWrongFormat(String, String),

    #[error("Cannot use number and airdrop feature at the same time")]
    CannotUseNumberAndAirdropFeatureAtTheSameTime,

    #[error("Airdrop total {0} is higher than available {1}")]
    AirdropTotalIsHigherThanAvailable(u64, u64),

    #[error("Failed to open AirDrop results file {0} with error {1}")]
    FailedToOpenAirDropResultsFile(String, String),

    #[error("Failed to parse AirDrop results file {0} with error {1}")]
    AirDropResultsFileWrongFormat(String, String),

    #[error("Overflow during sync of results and targets for address {0}")]
    OverflowDuringSyncOfResultsAndTargetsForAddress(String),
}
