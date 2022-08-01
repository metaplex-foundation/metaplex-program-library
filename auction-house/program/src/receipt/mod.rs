//! Create PDAs to to track the status and results of various Auction House actions.
use anchor_lang::{prelude::*, AnchorDeserialize, AnchorSerialize};

pub mod cancel_bid_receipt;
pub mod cancel_listing_receipt;
pub mod print_bid_receipt;
pub mod print_listing_receipt;
pub mod print_purchase_receipt;

pub use cancel_bid_receipt::*;
pub use cancel_listing_receipt::*;
pub use print_bid_receipt::*;
pub use print_listing_receipt::*;
pub use print_purchase_receipt::*;

pub const BID_RECEIPT_SIZE: usize = 8 + //key
32 + // trade_state
32 + // bookkeeper
32 + // auction_house
32 + // buyer
32 + // metadata
1 + 32 + // token_account
1 + 32 + // purchase_receipt
8 + // price
8 + // token_size
1 + // bump
1 + // trade_state_bump
8 + // created_at
1 + 8; // canceled_at

/// Receipt for a bid transaction.
#[account]
pub struct BidReceipt {
    pub trade_state: Pubkey,
    pub bookkeeper: Pubkey,
    pub auction_house: Pubkey,
    pub buyer: Pubkey,
    pub metadata: Pubkey,
    pub token_account: Option<Pubkey>,
    pub purchase_receipt: Option<Pubkey>,
    pub price: u64,
    pub token_size: u64,
    pub bump: u8,
    pub trade_state_bump: u8,
    pub created_at: i64,
    pub canceled_at: Option<i64>,
}

pub const LISTING_RECEIPT_SIZE: usize = 8 + //key
32 + // trade_state
32 + // bookkeeper
32 + // auction_house
32 + // seller
32 + // metadata
1 + 32 + // purchase_receipt
8 + // price
8 + // token_size
1 + // bump
1 + // trade_state_bump
8 + // created_at
1 + 8; // canceled_at;

/// Receipt for a listing transaction.
#[account]
pub struct ListingReceipt {
    pub trade_state: Pubkey,
    pub bookkeeper: Pubkey,
    pub auction_house: Pubkey,
    pub seller: Pubkey,
    pub metadata: Pubkey,
    pub purchase_receipt: Option<Pubkey>,
    pub price: u64,
    pub token_size: u64,
    pub bump: u8,
    pub trade_state_bump: u8,
    pub created_at: i64,
    pub canceled_at: Option<i64>,
}

pub const PURCHASE_RECEIPT_SIZE: usize = 8 + //key
32 + // bookkeeper
32 + // buyer
32 + // seller
32 + // auction_house
32 + // metadata
8 + // token_size
8 + // price
1 + // bump
8; // created_at

/// Receipt for a purchase transaction.
#[account]
pub struct PurchaseReceipt {
    pub bookkeeper: Pubkey,
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub auction_house: Pubkey,
    pub metadata: Pubkey,
    pub token_size: u64,
    pub price: u64,
    pub bump: u8,
    pub created_at: i64,
}
