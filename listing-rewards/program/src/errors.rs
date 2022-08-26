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
    #[msg("Cannot delete an already flushed out rewardable collection")]
    RewardableCollectionAlreadyDeleted,

    // 6009
    #[msg("The rewardable collection is already created/active")]
    RewardableCollectionAlreadyActive,

    // 6010
    #[msg("Math numerical overflow")]
    NumericalOverflowError,

    // 6011
    #[msg("The mints do not match")]
    MintMismatch,
}
