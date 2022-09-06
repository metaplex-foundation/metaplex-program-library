use anchor_lang::prelude::*;

#[error_code]
pub enum ListingRewardsError {
    // 6000
    #[msg("Bump seed not in hash map")]
    BumpSeedNotInHashMap,

    // 6001
    #[msg("Unauthorized signer")]
    SignerNotAuthorized,

    // 6002
    #[msg("Invalid collection maintainer")]
    InvalidCollectionMaintainer,

    // 6003
    #[msg("The NFT does not belong to a collection")]
    NFTMissingCollection,

    // 6004
    #[msg("The NFT does not match the rewardable collection")]
    NFTMismatchRewardableCollection,

    // 6005
    #[msg("The seller doesnt match the provided wallet")]
    SellerWalletMismatch,

    // 6006
    #[msg("The rewards were already claimed for this listing")]
    RewardsAlreadyClaimed,

    // 6007
    #[msg("The listings is not eligible for rewards yet")]
    IneligibaleForRewards,

    // 6008
    #[msg("Math numerical overflow")]
    NumericalOverflowError,

    // 6009
    #[msg("The mints do not match")]
    MintMismatch,

    // 6010
    #[msg("Listing and offer prices do not match")]
    PriceMismatch,

    // 6011
    #[msg("Cannot update price on an already cancelled listing")]
    ListingAlreadyCancelled,

    // 6012
    #[msg("Cannot update price on an already purchased listing")]
    ListingAlreadyPurchased,

    // 6013
    #[msg("Cannot update price on an already cancelled offer")]
    OfferAlreadyCancelled,

    // 6014
    #[msg("Cannot update price on an already purchased offer")]
    OfferAlreadyPurchased,
}
