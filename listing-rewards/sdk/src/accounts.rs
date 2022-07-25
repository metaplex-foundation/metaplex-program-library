use anchor_lang::prelude::Pubkey;

pub struct SellAccounts {
    pub wallet: Pubkey,
    pub listing: Pubkey,
    pub reward_center: Pubkey,
    pub rewardable_collection: Pubkey,
    pub token_account: Pubkey,
    pub metadata: Pubkey,
    pub authority: Pubkey,
    pub auction_house: Pubkey,
    pub seller_trade_state: Pubkey,
    pub free_seller_trade_state: Pubkey,
}

pub struct CreateOfferAccounts {
    pub wallet: Pubkey,
    pub offer: Pubkey,
    pub payment_account: Pubkey,
    pub rewardable_collection: Pubkey,
    pub transfer_authority: Pubkey,
    pub treasury_mint: Pubkey,
    pub token_account: Pubkey,
    pub metadata: Pubkey,
    pub authority: Pubkey,
    pub reward_center: Pubkey,
    pub auction_house: Pubkey,
    pub buyer_trade_state: Pubkey,
}
