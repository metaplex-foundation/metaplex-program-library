use anchor_lang::prelude::*;

#[error_code]
pub enum ListingRewardsError {
    // 6000
    #[msg("Bump seed not in hash map")]
    BumpSeedNotInHashMap,

    // 6001
    #[msg("Unauthorized signer")]
    SignerNotAuthorized,
}
