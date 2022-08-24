use anchor_lang::prelude::*;

#[error_code]
pub enum CandyError {
    #[msg("Account does not have correct owner")]
    IncorrectOwner,
    #[msg("Account is not initialized")]
    Uninitialized,
    #[msg("Mint Mismatch")]
    MintMismatch,
    #[msg("Index greater than length")]
    IndexGreaterThanLength,
    #[msg("Numerical overflow error")]
    NumericalOverflowError,
    #[msg("Can only provide up to 4 creators to candy machine (because candy machine is one)")]
    TooManyCreators,
    #[msg("Candy machine is empty")]
    CandyMachineEmpty,
    #[msg("Configs that are using hidden uris do not have config lines, they have a single hash representing hashed order")]
    HiddenSettingsConfigsDoNotHaveConfigLines,
    #[msg("Cannot change number of lines unless is a hidden config")]
    CannotChangeNumberOfLines,
    #[msg("Cannot switch to hidden settings after items available is greater than 0")]
    CannotSwitchToHiddenSettings,
    #[msg("Incorrect collection NFT authority")]
    IncorrectCollectionAuthority,
    #[msg("The metadata account has data in it, and this must be empty to mint a new NFT")]
    MetadataAccountMustBeEmpty,
    #[msg("Can't change collection settings after items have begun to be minted")]
    NoChangingCollectionDuringMint,
    #[msg("Retain authority must be true for Candy Machines with a collection set")]
    CandyCollectionRequiresRetainAuthority,
    #[msg("Value longer than expected maximum value")]
    ExceededLengthError,
    #[msg("Missing config lines settings")]
    MissingConfigLinesSettings,
    #[msg("Cannot increase the length in config lines settings")]
    CannotIncreaseLength,
    #[msg("Cannot switch from hidden settings")]
    CannotSwitchFromHiddenSettings,
    #[msg("Cannot change sequential index generation after items have begun to be minted")]
    CannotChangeSequentialIndexGeneration,
    #[msg("Collection public key mismatch")]
    CollectionKeyMismatch,
    #[msg("Missing collection accounts")]
    MissingCollectionAccounts,
    #[msg("Cannot change update authority if a collection mint is set")]
    CannotChangeUpdateAuthority,
}
