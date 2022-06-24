use anchor_lang::prelude::*;
use solana_program::clock::UnixTimestamp;

pub const BID_SIZE: usize = 8 + 1 + 32;
pub const LISTING_CONFIG_SIZE: usize = 8 + 1 + 8 + 8 + BID_SIZE + 1 + 8 + 8 + 4 + 4 + 1;

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub enum ListingConfigVersion {
    V0,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct Bid {
    pub version: ListingConfigVersion,
    pub amount: u64,
    pub buyer_trade_state: Pubkey,
}

#[account]
pub struct ListingConfig {
    pub version: ListingConfigVersion,
    pub start_time: UnixTimestamp,
    pub end_time: UnixTimestamp,
    pub highest_bid: Bid,
    pub bump: u8,
    pub reserve_price: u64,
    pub min_bid_increment: u64,
    pub time_ext_period: u32,
    pub time_ext_delta: u32,
    pub allow_high_bid_cancel: bool,
}
