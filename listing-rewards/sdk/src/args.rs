use anchor_lang::prelude::Pubkey;

pub struct CreateListingData {
    pub price: u64,
    pub token_size: u64,
    pub trade_state_bump: u8,
    pub free_trade_state_bump: u8,
}

pub struct UpdateListingData {
    pub new_price: u64,
}

pub struct CancelListingData {
    pub price: u64,
    pub token_size: u64,
}

pub struct CreateOfferData {
    pub buyer_price: u64,
    pub token_size: u64,
}

pub struct UpdateOfferData {
    pub new_buyer_price: u64,
}

pub struct CloseOfferData {
    pub buyer_price: u64,
    pub token_size: u64,
}

pub struct ExecuteSaleData {
    pub price: u64,
    pub token_size: u64,
    pub reward_mint: Pubkey,
}
