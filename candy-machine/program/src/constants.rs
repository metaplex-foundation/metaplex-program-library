use mpl_token_metadata::state::{
    MAX_CREATOR_LEN, MAX_CREATOR_LIMIT, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH,
};
use solana_program::pubkey::Pubkey;

pub const EXPIRE_OFFSET: i64 = 10 * 60;
pub const BLOCK_HASHES: Pubkey =
    solana_program::pubkey!("SysvarRecentB1ockHashes11111111111111111111");
pub const BOT_FEE: u64 = 10000000;
pub const PREFIX: &str = "candy_machine";
pub const COLLECTIONS_FEATURE_INDEX: usize = 0;
pub const CONFIG_LINE_SIZE: usize = 4 + MAX_NAME_LENGTH + 4 + MAX_URI_LENGTH;
pub const COLLECTION_PDA_SIZE: usize = 8 + 64;
pub const GUMDROP_ID: Pubkey =
    solana_program::pubkey!("gdrpGjVffourzkdDRrQmySw4aTHr8a3xmQzzxSwFD1a");
pub const CUPCAKE_ID: Pubkey =
    solana_program::pubkey!("cakeGJxEdGpZ3MJP8sM3QypwzuzZpko1ueonUQgKLPE");
pub const A_TOKEN: Pubkey = solana_program::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
pub const CONFIG_ARRAY_START: usize = 8 + // key
    32 + // authority
    32 + //wallet
    33 + // token mint
    4 + 6 + // uuid
    8 + // price
    8 + // items available
    9 + // go live
    10 + // end settings
    4 + MAX_SYMBOL_LENGTH + // u32 len + symbol
    2 + // seller fee basis points
    4 + MAX_CREATOR_LIMIT*MAX_CREATOR_LEN + // optional + u32 len + actual vec
    8 + //max supply
    1 + // is mutable
    1 + // retain authority
    1 + // option for hidden setting
    4 + MAX_NAME_LENGTH + // name length,
    4 + MAX_URI_LENGTH + // uri length,
    32 + // hash
    4 +  // max number of lines;
    8 + // items redeemed
    1 + // whitelist option
    1 + // whitelist mint mode
    1 + // allow presale
    9 + // discount price
    32 + // mint key for whitelist
    1 + 32 + 1; // gatekeeper
