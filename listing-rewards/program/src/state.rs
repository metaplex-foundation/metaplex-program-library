use anchor_lang::prelude::*;

#[account]
pub struct RewardableCollection {
    /// is Initialized
    pub is_initialized: bool,
    /// rewardable collection maintainer
    pub maintainer: Pubkey,
    /// the mint address of the collection
    pub collection: Pubkey,
    /// the address of the associated reward center
    pub reward_center: Pubkey,
    /// the pda bump
    pub bump: u8,
    /// deleted at timestamp
    pub deleted_at: Option<i64>,
}

impl RewardableCollection {
    pub fn size() -> usize {
        8 + // deliminator
      1 + // is_initialized
      32 + // maintainer
      32 + // collection
      32 + // reward_center
      1 + // pda bump
      1 + 8 // deleted_at
    }
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Debug)]
pub struct ListingRewardRules {
    /// time a listing must be up before is eligable for a rewards in seconds
    pub warmup_seconds: i64,
    /// number of tokens to reward for listing
    pub reward_payout: u64,
    // Basis Points to determine reward ratio for seller
    pub seller_reward_payout_basis_points: u16,
    // Payout Divider for determining reward distribution to seller/buyer
    pub payout_divider: u16,
}

#[account]
#[derive(Debug)]
pub struct RewardCenter {
    /// the mint of the token used as rewards
    pub token_mint: Pubkey,
    /// the auction house associated to the reward center
    pub auction_house: Pubkey,
    /// the oracle allowed to adjust rewardable collections
    pub collection_oracle: Option<Pubkey>,
    /// rules for listing rewards
    pub listing_reward_rules: ListingRewardRules,
    /// the bump of the pda
    pub bump: u8,
}

impl RewardCenter {
    pub fn size() -> usize {
        8 + // deliminator
        32 + // token_mint
        32 + // auction_house
        1 + 32 + // optional collection oracle
        8 + 8 + 2 + 2 + // listing reward rules
        1 // bump
    }
}

#[account]
pub struct Listing {
    pub reward_center: Pubkey,
    pub seller: Pubkey,
    pub metadata: Pubkey,
    pub price: u64,
    pub token_size: u64,
    pub bump: u8,
    pub created_at: i64,
    pub canceled_at: Option<i64>,
    pub purchased_at: Option<i64>,
    pub rewardable_collection: Pubkey,
    pub reward_redeemed_at: Option<i64>,
}

impl Listing {
    pub fn size() -> usize {
        8 + // deliminator
        32 + // reward_center
        32 + // seller
        32 + // metadata
        8 + // price
        8 + // token_size
        1 + // bump
        8 + // created_at
        1 + 8 + // canceled_at
        1 + 8 + // purchased_at
        32 + // rewardable_collection
        1 + 8 // reward_redeemed_at
    }
}

#[account]
pub struct Offer {
    pub reward_center: Pubkey,
    pub buyer: Pubkey,
    pub metadata: Pubkey,
    pub price: u64,
    pub token_size: u64,
    pub bump: u8,
    pub created_at: i64,
    pub canceled_at: Option<i64>,
    pub purchased_at: Option<i64>,
    pub rewardable_collection: Pubkey,
}

impl Offer {
    pub fn size() -> usize {
        8 + // deliminator
        32 + // reward_center
        32 + // buyer
        32 + // metadata
        8 + // price
        8 + // token_size
        1 + // bump
        8 + // created_at
        1 + 8 + // canceled_at
        1 + 8 + // purchased_at
        32 // rewardable_collection
    }
}
