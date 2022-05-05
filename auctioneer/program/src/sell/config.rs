use anchor_lang::prelude::*;
use solana_program::clock::UnixTimestamp;

pub const BID_SIZE: usize = 8 + 32;
pub const LISTING_CONFIG_SIZE: usize = 8 + 8 + 8 + BID_SIZE + 1;

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct Bid {
    pub amount: u64,
    pub buyer_trade_state: Pubkey,
}

#[account]
pub struct ListingConfig {
    pub start_time: UnixTimestamp,
    pub end_time: UnixTimestamp,
    pub highest_bid: Bid,
    pub bump: u8,
}
