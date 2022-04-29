use anchor_lang::prelude::*;
use arrayref::array_ref;
use mpl_token_metadata::state::{
    MAX_CREATOR_LEN, MAX_CREATOR_LIMIT, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH,
};
use solana_program::pubkey::Pubkey;
use std::cell::RefMut;

/// Candy machine state and config data.
#[account]
#[derive(Default)]
pub struct CandyMachine {
    pub authority: Pubkey,
    pub wallet: Pubkey,
    pub token_mint: Option<Pubkey>,
    pub items_redeemed: u64,
    pub data: CandyMachineData,
    // there's a borsh vec u32 denoting how many actual lines of data there are currently (eventually equals items available)
    // There is actually lines and lines of data after this but we explicitly never want them deserialized.
    // here there is a borsh vec u32 indicating number of bytes in bitmask array.
    // here there is a number of bytes equal to ceil(max_number_of_lines/8) and it is a bit mask used to figure out when to increment borsh vec u32
}
pub const COLLECTION_PDA_SIZE: usize = 8 + 64;
/// Collection PDA account
#[account]
#[derive(Default, Debug)]
pub struct CollectionPDA {
    pub mint: Pubkey,
    pub candy_machine: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WhitelistMintSettings {
    pub mode: WhitelistMintMode,
    pub mint: Pubkey,
    pub presale: bool,
    pub discount_price: Option<u64>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum WhitelistMintMode {
    // Only captcha uses the bytes, the others just need to have same length
    // for front end borsh to not crap itself
    // Holds the validation window
    BurnEveryTime,
    NeverBurn,
}

/// Candy machine settings data.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct CandyMachineData {
    pub uuid: String,
    pub price: u64,
    /// The symbol for the asset
    pub symbol: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000)
    pub seller_fee_basis_points: u16,
    pub max_supply: u64,
    pub is_mutable: bool,
    pub retain_authority: bool,
    pub go_live_date: Option<i64>,
    pub end_settings: Option<EndSettings>,
    pub creators: Vec<Creator>,
    pub hidden_settings: Option<HiddenSettings>,
    pub whitelist_mint_settings: Option<WhitelistMintSettings>,
    pub items_available: u64,
    /// If [`Some`] requires gateway tokens on mint
    pub gatekeeper: Option<GatekeeperConfig>,
}

/// Configurations options for the gatekeeper.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct GatekeeperConfig {
    /// The network for the gateway token required
    pub gatekeeper_network: Pubkey,
    /// Whether or not the token should expire after minting.
    /// The gatekeeper network must support this if true.
    pub expire_on_use: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum EndSettingType {
    Date,
    Amount,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct EndSettings {
    pub end_setting_type: EndSettingType,
    pub number: u64,
}

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

/// Hidden Settings for large mints used with offline data.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct HiddenSettings {
    pub name: String,
    pub uri: String,
    pub hash: [u8; 32],
}

pub fn get_config_count(data: &RefMut<&mut [u8]>) -> Result<usize> {
    return Ok(u32::from_le_bytes(*array_ref![data, CONFIG_ARRAY_START, 4]) as usize);
}

/// Individual config line for storing NFT data pre-mint.
pub const CONFIG_LINE_SIZE: usize = 4 + MAX_NAME_LENGTH + 4 + MAX_URI_LENGTH;
#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ConfigLine {
    pub name: String,
    /// URI pointing to JSON representing the asset
    pub uri: String,
}

// Unfortunate duplication of token metadata so that IDL picks it up.

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    // In percentages, NOT basis points ;) Watch out!
    pub share: u8,
}
