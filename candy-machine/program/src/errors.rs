use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Enum representing errors in the Candy Machine
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum CandyError {
    #[error("Account does not have correct owner!")]
    IncorrectOwner,
    #[error("Account is not initialized!")]
    Uninitialized,
    #[error("Mint Mismatch!")]
    MintMismatch,
    #[error("Index greater than length!")]
    IndexGreaterThanLength,
    #[error("Numerical overflow error!")]
    NumericalOverflowError,
    #[error("Can only provide up to 4 creators to candy machine (because candy machine is one)!")]
    TooManyCreators,
    #[error("Uuid must be exactly of 6 length")]
    UuidMustBeExactly6Length,
    #[error("Not enough tokens to pay for this minting")]
    NotEnoughTokens,
    #[error("Not enough SOL to pay for this minting")]
    NotEnoughSOL,
    #[error("Token transfer failed")]
    TokenTransferFailed,
    #[error("Candy machine is empty!")]
    CandyMachineEmpty,
    #[error("Candy machine is not live!")]
    CandyMachineNotLive,
    #[error("Configs that are using hidden uris do not have config lines, they have a single hash representing hashed order")]
    HiddenSettingsConfigsDoNotHaveConfigLines,
    #[error("Cannot change number of lines unless is a hidden config")]
    CannotChangeNumberOfLines,
    #[error("Derived key invalid")]
    DerivedKeyInvalid,
    #[error("Public key mismatch")]
    PublicKeyMismatch,
    #[error("No whitelist token present")]
    NoWhitelistToken,
    #[error("Token burn failed")]
    TokenBurnFailed,
    #[error("Missing gateway app when required")]
    GatewayAppMissing,
    #[error("Missing gateway token when required")]
    GatewayTokenMissing,
    #[error("Invalid gateway token expire time")]
    GatewayTokenExpireTimeInvalid,
    #[error("Missing gateway network expire feature when required")]
    NetworkExpireFeatureMissing,
    #[error("Unable to find an unused config line near your random number index")]
    CannotFindUsableConfigLine,
    #[error("Invalid string")]
    InvalidString,
    #[error("Suspicious transaction detected")]
    SuspiciousTransaction,
    #[error("Cannot Switch to Hidden Settings after items available is greater than 0")]
    CannotSwitchToHiddenSettings,
    #[error("Incorrect SlotHashes PubKey")]
    IncorrectSlotHashesPubkey,
    #[error("Incorrect collection NFT authority")]
    IncorrectCollectionAuthority,
    #[error("Collection PDA address is invalid")]
    MismatchedCollectionPDA,
    #[error("Provided mint account doesn't match collection PDA mint")]
    MismatchedCollectionMint,
    #[error("Slot hashes Sysvar is empty")]
    SlotHashesEmpty,
    #[error("The metadata account has data in it, and this must be empty to mint a new NFT")]
    MetadataAccountMustBeEmpty,
    #[error("Missing set collection during mint IX for Candy Machine with collection set")]
    MissingSetCollectionDuringMint,
    #[error("Can't change collection settings after items have begun to be minted")]
    NoChangingCollectionDuringMint,
    #[error("Retain authority must be true for Candy Machines with a collection set")]
    CandyCollectionRequiresRetainAuthority,
    #[error("Error within Gateway program")]
    GatewayProgramError,
    #[error("Can't change freeze settings after items have begun to be minted. You can only disable.")]
    NoChangingFreezeDuringMint,
    #[error("Can't change authority while collection is enabled. Disable collection first.")]
    NoChangingAuthorityWithCollection,
    #[error("Can't change token while freeze is enabled. Disable freeze first.")]
    NoChangingTokenWithFreeze,
    #[error("Cannot thaw NFT unless all NFTs are minted or Candy Machine authority enables thawing")]
    InvalidThawNft,
    #[error("The number of remaining accounts passed in doesn't match the Candy Machine settings")]
    IncorrectRemainingAccountsLen,
    #[error("FreezePDA ATA needs to be passed in if token mint is enabled.")]
    MissingFreezeAta,
    #[error("Incorrect freeze ATA address.")]
    IncorrectFreezeAta,
    #[error("FreezePDA doesn't belong to this Candy Machine.")]
    FreezePDAMismatch,
    #[error("Freeze time can't be longer than MAX_FREEZE_TIME.")]
    EnteredFreezeIsMoreThanMaxFreeze,
    #[error("Can't withdraw Candy Machine while freeze is active. Disable freeze first.")]
    NoWithdrawWithFreeze,
    #[error("Can't withdraw Candy Machine while frozen funds need to be redeemed. Unlock funds first.")]
    NoWithdrawWithFrozenFunds,
    #[error("Missing required remaining accounts for remove_freeze with token mint.")]
    MissingRemoveFreezeTokenAccounts,
    #[error("Can't withdraw SPL Token from freeze PDA into itself")]
    InvalidFreezeWithdrawTokenAddress,
    #[error("Can't unlock funds while NFTs are still frozen. Run thaw on all NFTs first.")]
    NoUnlockWithNFTsStillFrozen,
    #[error("Setting a sized collection requires the collection metadata to be mutable.")]
    SizedCollectionMetadataMustBeMutable,
    #[error("Cannot remove Hidden Settings.")]
    CannotSwitchFromHiddenSettings,
    #[error("Invalid Metadata Account")]
    InvalidMetadataAccount,
}
