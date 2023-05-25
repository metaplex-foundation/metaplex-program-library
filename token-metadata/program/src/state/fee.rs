use super::*;

pub(crate) const FEE_AUTHORITY: Pubkey = pubkey!("Levytx9LLPzAtDJJD7q813Zsm8zg9e1pb53mGxTKpD7");

// base fee level, 0.001 SOL
pub const BASE_FEE: u64 = 1_000_000;

// create_metadata_accounts
pub const CREATE_FEE: u64 = 10 * BASE_FEE;
pub const UPDATE_FEE: u64 = 2 * BASE_FEE;

pub const FEE_FLAG_SET: u8 = 1;
pub const FEE_FLAG_CLEARED: u8 = 0;
