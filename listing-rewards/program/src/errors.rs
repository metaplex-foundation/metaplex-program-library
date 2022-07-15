use anchor_lang::prelude::*;

#[error_code]
pub enum ListingRewardsError {
    // 6000
    #[msg("Bump seed not in hash map")]
    BumpSeedNotInHashMap,

    // 6001
    #[msg("Unauthorized signer")]
    SignerNotAuthorized,

    // 6003
    #[msg("Invalid collection maintainer")]
    InvalidCollectionMaintainer,

    // 6004
    #[msg("The NFT does not belong to a collection")]
    NFTMissingCollection,

    // 6005
    #[msg("The NFT does not match the rewardable collection")]
    NFTMismatchRewardableCollection
}
