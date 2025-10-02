use anchor_lang::prelude::*;

#[error_code]
pub enum CandyError {
    #[msg("Account does not have correct owner!")]
    IncorrectOwner,
    #[msg("Account is not initialized!")]
    Uninitialized,
    #[msg("Mint Mismatch!")]
    MintMismatch,
    #[msg("Index greater than length!")]
    IndexGreaterThanLength,
    #[msg("Numerical overflow error!")]
    NumericalOverflowError,
    #[msg("Can only provide up to 4 creators to candy machine (because candy machine is one)!")]
    TooManyCreators,
    #[msg("Uuid must be exactly of 6 length")]
    UuidMustBeExactly6Length,
    #[msg("Not enough tokens to pay for this minting")]
    NotEnoughTokens,
    #[msg("Not enough SOL to pay for this minting")]
    NotEnoughSOL,
    #[msg("Token transfer failed")]
    TokenTransferFailed,
    #[msg("Candy machine is empty!")]
    CandyMachineEmpty,
    #[msg("Candy machine is not live!")]
    CandyMachineNotLive,
    #[msg("Configs that are using hidden uris do not have config lines, they have a single hash representing hashed order")]
    HiddenSettingsConfigsDoNotHaveConfigLines,
    #[msg("Cannot change number of lines unless is a hidden config")]
    CannotChangeNumberOfLines,
    #[msg("Derived key invalid")]
    DerivedKeyInvalid,
    #[msg("Public key mismatch")]
    PublicKeyMismatch,
    #[msg("No whitelist token present")]
    NoWhitelistToken,
    #[msg("Token burn failed")]
    TokenBurnFailed,
    #[msg("Missing gateway app when required")]
    GatewayAppMissing,
    #[msg("Missing gateway token when required")]
    GatewayTokenMissing,
    #[msg("Invalid gateway token expire time")]
    GatewayTokenExpireTimeInvalid,
    #[msg("Missing gateway network expire feature when required")]
    NetworkExpireFeatureMissing,
    #[msg("Unable to find an unused config line near your random number index")]
    CannotFindUsableConfigLine,
    #[msg("Invalid string")]
    InvalidString,
    #[msg("Suspicious transaction detected")]
    SuspiciousTransaction,
    #[msg("Cannot Switch to Hidden Settings after items available is greater than 0")]
    CannotSwitchToHiddenSettings,
    #[msg("Incorrect SlotHashes PubKey")]
    IncorrectSlotHashesPubkey,
    #[msg("Incorrect collection NFT authority")]
    IncorrectCollectionAuthority,
    #[msg("Collection PDA address is invalid")]
    MismatchedCollectionPDA,
    #[msg("Provided mint account doesn't match collection PDA mint")]
    MismatchedCollectionMint,
    #[msg("Slot hashes Sysvar is empty")]
    SlotHashesEmpty,
    #[msg("The metadata account has data in it, and this must be empty to mint a new NFT")]
    MetadataAccountMustBeEmpty,
    #[msg("Missing set collection during mint IX for Candy Machine with collection set")]
    MissingSetCollectionDuringMint,
    #[msg("Can't change collection settings after items have begun to be minted")]
    NoChangingCollectionDuringMint,
    #[msg("Retain authority must be true for Candy Machines with a collection set")]
    CandyCollectionRequiresRetainAuthority,
    #[msg("Error within Gateway program")]
    GatewayProgramError,
    #[msg(
        "Can't change freeze settings after items have begun to be minted. You can only disable."
    )]
    NoChangingFreezeDuringMint,
    #[msg("Can't change authority while collection is enabled. Disable collection first.")]
    NoChangingAuthorityWithCollection,
    #[msg("Can't change token while freeze is enabled. Disable freeze first.")]
    NoChangingTokenWithFreeze,
    #[msg("Cannot thaw NFT unless all NFTs are minted or Candy Machine authority enables thawing")]
    InvalidThawNft,
    #[msg("The number of remaining accounts passed in doesn't match the Candy Machine settings")]
    IncorrectRemainingAccountsLen,
    #[msg("FreezePDA ATA needs to be passed in if token mint is enabled.")]
    MissingFreezeAta,
    #[msg("Incorrect freeze ATA address.")]
    IncorrectFreezeAta,
    #[msg("FreezePDA doesn't belong to this Candy Machine.")]
    FreezePDAMismatch,
    #[msg("Freeze time can't be longer than MAX_FREEZE_TIME.")]
    EnteredFreezeIsMoreThanMaxFreeze,
    #[msg("Can't withdraw Candy Machine while freeze is active. Disable freeze first.")]
    NoWithdrawWithFreeze,
    #[msg(
        "Can't withdraw Candy Machine while frozen funds need to be redeemed. Unlock funds first."
    )]
    NoWithdrawWithFrozenFunds,
    #[msg("Missing required remaining accounts for remove_freeze with token mint.")]
    MissingRemoveFreezeTokenAccounts,
    #[msg("Can't withdraw SPL Token from freeze PDA into itself")]
    InvalidFreezeWithdrawTokenAddress,
    #[msg("Can't unlock funds while NFTs are still frozen. Run thaw on all NFTs first.")]
    NoUnlockWithNFTsStillFrozen,
    #[msg("Setting a sized collection requires the collection metadata to be mutable.")]
    SizedCollectionMetadataMustBeMutable,
    #[msg("Cannot remove Hidden Settings.")]
    CannotSwitchFromHiddenSettings,
    #[msg("Invalid Metadata Account")]
    InvalidMetadataAccount,
}
