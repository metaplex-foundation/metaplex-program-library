use anchor_lang::prelude::Pubkey;

pub struct CreateListingAccounts {
    pub wallet: Pubkey,
    pub listing: Pubkey,
    pub reward_center: Pubkey,
    pub token_account: Pubkey,
    pub metadata: Pubkey,
    pub authority: Pubkey,
    pub auction_house: Pubkey,
    pub seller_trade_state: Pubkey,
    pub free_seller_trade_state: Pubkey,
}

pub struct CancelListingAccounts {
    pub wallet: Pubkey,
    pub listing: Pubkey,
    pub reward_center: Pubkey,
    pub metadata: Pubkey,
    pub token_account: Pubkey,
    pub authority: Pubkey,
    pub auction_house: Pubkey,
    pub token_mint: Pubkey,
    pub treasury_mint: Pubkey,
}

pub struct UpdateListingAccounts {
    pub wallet: Pubkey,
    pub metadata: Pubkey,
    pub token_account: Pubkey,
    pub auction_house: Pubkey,
}

pub struct CreateOfferAccounts {
    pub wallet: Pubkey,
    pub payment_account: Pubkey,
    pub transfer_authority: Pubkey,
    pub treasury_mint: Pubkey,
    pub token_mint: Pubkey,
    pub token_account: Pubkey,
    pub metadata: Pubkey,
    pub authority: Pubkey,
    pub reward_center: Pubkey,
    pub auction_house: Pubkey,
}

pub struct UpdateOfferAccounts {
    pub wallet: Pubkey,
    pub buyer_token_account: Pubkey,
    pub transfer_authority: Pubkey,
    pub treasury_mint: Pubkey,
    pub token_account: Pubkey,
    pub metadata: Pubkey,
    pub authority: Pubkey,
    pub reward_center: Pubkey,
    pub auction_house: Pubkey,
}

pub struct CloseOfferAccounts {
    pub wallet: Pubkey,
    pub receipt_account: Pubkey,
    pub treasury_mint: Pubkey,
    pub token_mint: Pubkey,
    pub token_account: Pubkey,
    pub metadata: Pubkey,
    pub authority: Pubkey,
    pub reward_center: Pubkey,
    pub auction_house: Pubkey,
}

pub struct ExecuteSaleAccounts {
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub authority: Pubkey,
    pub auction_house: Pubkey,
    pub treasury_mint: Pubkey,
    pub token_mint: Pubkey,
    pub token_account: Pubkey,
    pub metadata: Pubkey,
    pub seller_payment_receipt_account: Pubkey,
    pub buyer_receipt_token_account: Pubkey,
}
