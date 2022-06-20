pub use anchor_lang::{prelude::*, AccountDeserialize, InstructionData, ToAccountMetas};

pub use mpl_auction_house::{
    pda::{find_auctioneer_pda, find_bid_receipt_address, find_listing_receipt_address},
    receipt::{BidReceipt, ListingReceipt},
    AuctionHouse, Auctioneer, AuthorityScope,
};
pub use mpl_testing_utils::{
    assert_error, assert_transport_error, solana::airdrop, utils::Metadata,
};
pub use spl_associated_token_account::get_associated_token_address;
pub use spl_token;

pub use solana_program_test::*;
pub use solana_sdk::{
    instruction::{Instruction, InstructionError},
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};
pub use std::assert_eq;

pub const HAS_ONE_CONSTRAINT_VIOLATION: u32 = 2001;
pub const INVALID_SEEDS: u32 = 2006;

pub const MISSING_AUCTIONEER_SCOPE: u32 = 6029;
pub const NO_AUCTIONEER_PROGRAM_SET: u32 = 6031;
pub const INVALID_SEEDS_OR_NO_DELEGATE: u32 = 6036;
pub const TOO_MANY_SCOPES: u32 = 6032;
pub const BUYER_TRADE_STATE_NOT_VALID: u32 = 6037;
pub const NOT_ENOUGH_TOKENS_AVAIL_FOR_PURCHASE: u32 = 6039;
pub const PARTIAL_BUY_PRICE_MISMATCH: u32 = 6040;
pub const MISSING_ELEMENTS_NEEDED_FOR_PARTIAL_BUY: u32 = 6038;

pub const TEN_SOL: u64 = 10_000_000_000;
pub const ONE_SOL: u64 = 1_000_000_000;
