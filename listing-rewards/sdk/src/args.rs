use anchor_lang::prelude::Pubkey;

pub struct SellData {
    pub price: u64,
    pub token_size: u64,
    pub trade_state_bump: u8,
    pub free_trade_state_bump: u8,
}

pub struct CreateOfferData {
    pub collection: Pubkey,
    pub trade_state_bump: u8,
    pub buyer_price: u64,
    pub token_size: u64,
}
