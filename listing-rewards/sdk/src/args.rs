pub struct CreateListingData {
    pub price: u64,
    pub token_size: u64,
    pub trade_state_bump: u8,
    pub free_trade_state_bump: u8,
}

pub struct CancelListingData {
    pub price: u64,
    pub token_size: u64,
}

pub struct CreateOfferData {
    pub buyer_price: u64,
    pub token_size: u64,
}

pub struct CloseOfferData {
    pub buyer_price: u64,
    pub token_size: u64,
}
