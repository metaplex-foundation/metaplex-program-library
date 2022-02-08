pub use mpl_token_metadata::state::{
    MAX_CREATOR_LEN, MAX_CREATOR_LIMIT, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH,
};

pub const METAPLEX_PROGRAM_ID: &'static str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
pub const CANDY_MACHINE_V2: &'static str = "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ";
pub const CIVIC: &'static str = "gatem74V238djXdzWnJf94Wo1DcnuGkfijbf3AuBhfs";

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
1 + 32 + 1 // gatekeeper
;

pub const CONFIG_LINE_SIZE: usize = 4 + MAX_NAME_LENGTH + 4 + MAX_URI_LENGTH;
pub const CONFIG_CHUNK_SIZE: usize = 10;
pub const CONFIG_NAME_OFFSET: usize = 2;
pub const CONFIG_URI_OFFSET: usize = 40;
pub const STRING_LEN_SIZE: usize = 4;

pub const MINT_LAYOUT: u64 = 82;
