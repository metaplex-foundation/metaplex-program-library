use anchor_lang::prelude::*;
use mpl_token_metadata::state::{
    MAX_CREATOR_LEN, MAX_CREATOR_LIMIT, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH,
};
use solana_program::pubkey::Pubkey;

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

/// Hidden Settings for large mints used with offline data.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct HiddenSettings {
    pub name: String,
    pub uri: String,
    pub hash: [u8; 32],
}

// Unfortunate duplication of token metadata so that IDL picks it up.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    // In percentages, NOT basis points ;) Watch out!
    pub share: u8,
}

/// Individual config line for storing NFT data pre-mint.
pub const CONFIG_LINE_SIZE: usize = 4 + MAX_NAME_LENGTH + 4 + MAX_URI_LENGTH;
#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ConfigLine {
    pub name: String,
    /// URI pointing to JSON representing the asset
    pub uri: String,
}

pub trait CandyMachineAccessor {
    fn get_authority(&self) -> Pubkey;
    fn set_authority(&mut self, authority: Pubkey);
    fn get_wallet(&self) -> Pubkey;
    fn set_wallet(&mut self, wallet: Pubkey);
    fn get_token_mint(&self) -> Option<Pubkey>;
    fn set_token_mint(&mut self, token_mint: Option<Pubkey>);
    fn get_items_redeemed(&self) -> u64;
    fn set_items_redeemed(&mut self, items_redeemed: u64);
    // fn get_uuid(&self) -> String;
    // fn set_uuid(&mut self, uuid: String);
    fn get_price(&self) -> u64;
    fn set_price(&mut self, price: u64);
    fn get_symbol(&self) -> String;
    fn set_symbol(&mut self, symbol: String);
    fn get_seller_fee_basis_points(&self) -> u16;
    fn set_seller_fee_basis_points(&mut self, seller_fee_basis_points: u16);
    fn get_max_supply(&self) -> u64;
    fn set_max_supply(&mut self, max_supply: u64);
    fn get_is_mutable(&self) -> bool;
    fn set_is_mutable(&mut self, is_mutable: bool);
    fn get_retain_authority(&self) -> bool;
    fn set_retain_authority(&mut self, retain_authority: bool);
    fn get_go_live_date(&self) -> Option<i64>;
    fn set_go_live_date(&mut self, go_live_date: Option<i64>);
    fn get_end_settings(&self) -> Option<EndSettings>;
    fn set_end_settings(&mut self, end_settings: Option<EndSettings>);
    fn set_creators(&self) -> Vec<Creator>;
    fn get_creators(&mut self, creators: Vec<Creator>);
    fn get_hidden_settings(&self) -> Option<HiddenSettings>;
    fn set_hidden_settings(&mut self, hidden_settings: Option<HiddenSettings>);
    fn get_whitelist_mint_settings(&self) -> Option<WhitelistMintSettings>;
    fn set_whitelist_mint_settings(
        &mut self,
        whitelist_mint_settings: Option<WhitelistMintSettings>,
    );
    fn get_items_available(&self) -> u64;
    fn set_items_available(&mut self, items_available: u64);
    fn get_gatekeeper(&self) -> Option<GatekeeperConfig>;
    fn set_gatekeeper(&mut self, gatekeeper: Option<GatekeeperConfig>);
    fn get_collection_mint(&self) -> Option<Pubkey>;
    fn get_config_array_start(&self) -> usize {
        CONFIG_ARRAY_START
    }
}

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

#[account]
#[derive(Default)]
pub struct CandyMachineV2 {
    pub authority: Pubkey,
    pub wallet: Pubkey,
    pub token_mint: Option<Pubkey>,
    pub items_redeemed: u64,
    pub data: CandyMachineDataV2,
}

#[non_exhaustive]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct CandyMachineDataV2 {
    pub price: u64,
    pub items_available: u64,
    pub go_live_date: Option<i64>,
    /// Candy machine configuration options
    pub candy_options: CandyOptions,
    /// Options that are used when the NFTs are minted
    pub nft_settings: NftSettings,
    /// If [`Some`] requires gateway tokens on mint
    pub config_line_settings: ConfigLineSettings,
}

#[non_exhaustive]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct NftSettings {
    /// How many editions can be minted from each NFT (
    pub max_supply: u64,
    /// The symbol for the asset
    pub symbol: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000)
    pub seller_fee_basis_points: u16,
    /// The creators for the NFT
    pub creators: Vec<Creator>,
    /// The MCC for all the NFTs
    pub collection: Option<Pubkey>,
    /// Whether or not the authority of the NFT is transferred to the minter
    pub retain_authority: bool,
    /// Whether or not the NFT can be changed after mint
    pub is_mutable: bool,
}

#[non_exhaustive]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct CandyOptions {
    pub whitelist_mint_settings: Option<WhitelistMintSettings>,
    pub gatekeeper: Option<GatekeeperConfig>,
    pub end_settings: Option<EndSettings>,
    pub hidden_settings: Option<HiddenSettings>,
    // Plans to add more to this for space savings
}

#[non_exhaustive]
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ConfigLineSettings {
    Default,
    // Plans to add more to this for space savings
}

impl Default for ConfigLineSettings {
    fn default() -> Self {
        ConfigLineSettings::Default
    }
}

pub const COLLECTION_PDA_SIZE: usize = 8 + 64;
/// Collection PDA account
#[account]
#[derive(Default, Debug)]
pub struct CollectionPDA {
    pub mint: Pubkey,
    pub candy_machine: Pubkey,
}

impl CandyMachineAccessor for CandyMachine {
    fn get_authority(&self) -> Pubkey {
        self.authority
    }

    fn set_authority(&mut self, authority: Pubkey) {
        self.authority = authority;
    }

    fn get_wallet(&self) -> Pubkey {
        self.wallet
    }

    fn set_wallet(&mut self, wallet: Pubkey) {
        self.wallet = wallet;
    }

    fn get_token_mint(&self) -> Option<Pubkey> {
        self.token_mint
    }

    fn set_token_mint(&mut self, token_mint: Option<Pubkey>) {
        self.token_mint = token_mint;
    }

    fn get_items_redeemed(&self) -> u64 {
        self.items_redeemed
    }

    fn set_items_redeemed(&mut self, items_redeemed: u64) {
        self.items_redeemed = items_redeemed;
    }

    fn get_price(&self) -> u64 {
        self.data.price
    }

    fn set_price(&mut self, price: u64) {
        self.data.price = price;
    }

    fn get_symbol(&self) -> String {
        self.data.symbol.clone()
    }

    fn set_symbol(&mut self, symbol: String) {
        self.data.symbol = symbol;
    }

    fn get_seller_fee_basis_points(&self) -> u16 {
        self.data.seller_fee_basis_points
    }

    fn set_seller_fee_basis_points(&mut self, seller_fee_basis_points: u16) {
        self.data.seller_fee_basis_points = seller_fee_basis_points;
    }

    fn get_max_supply(&self) -> u64 {
        self.data.max_supply
    }

    fn set_max_supply(&mut self, max_supply: u64) {
        self.data.max_supply = max_supply;
    }

    fn get_is_mutable(&self) -> bool {
        self.data.is_mutable
    }

    fn set_is_mutable(&mut self, is_mutable: bool) {
        self.data.is_mutable = is_mutable;
    }

    fn get_retain_authority(&self) -> bool {
        self.data.retain_authority
    }

    fn set_retain_authority(&mut self, retain_authority: bool) {
        self.data.retain_authority = retain_authority;
    }

    fn get_go_live_date(&self) -> Option<i64> {
        self.data.go_live_date
    }

    fn set_go_live_date(&mut self, go_live_date: Option<i64>) {
        self.data.go_live_date = go_live_date;
    }

    fn get_end_settings(&self) -> Option<EndSettings> {
        self.data.end_settings.clone()
    }

    fn set_end_settings(&mut self, end_settings: Option<EndSettings>) {
        self.data.end_settings = end_settings;
    }

    fn set_creators(&self) -> Vec<Creator> {
        self.data.creators.clone()
    }

    fn get_creators(&mut self, creators: Vec<Creator>) {
        self.data.creators = creators;
    }

    fn get_hidden_settings(&self) -> Option<HiddenSettings> {
        self.data.hidden_settings.clone()
    }

    fn set_hidden_settings(&mut self, hidden_settings: Option<HiddenSettings>) {
        self.data.hidden_settings = hidden_settings;
    }

    fn get_whitelist_mint_settings(&self) -> Option<WhitelistMintSettings> {
        self.data.whitelist_mint_settings.clone()
    }

    fn set_whitelist_mint_settings(
        &mut self,
        whitelist_mint_settings: Option<WhitelistMintSettings>,
    ) {
        self.data.whitelist_mint_settings = whitelist_mint_settings;
    }

    fn get_items_available(&self) -> u64 {
        self.data.items_available
    }

    fn set_items_available(&mut self, items_available: u64) {
        self.data.items_available = items_available;
    }

    fn get_gatekeeper(&self) -> Option<GatekeeperConfig> {
        self.data.gatekeeper.clone()
    }

    fn set_gatekeeper(&mut self, gatekeeper: Option<GatekeeperConfig>) {
        self.data.gatekeeper = gatekeeper;
    }

    fn get_collection_mint(&self) -> Option<Pubkey> {
        None
    }
}

impl CandyMachineAccessor for CandyMachineV2 {
    fn get_authority(&self) -> Pubkey {
        self.authority
    }

    fn set_authority(&mut self, authority: Pubkey) {
        self.authority = authority;
    }

    fn get_wallet(&self) -> Pubkey {
        self.wallet
    }

    fn set_wallet(&mut self, wallet: Pubkey) {
        self.wallet = wallet;
    }

    fn get_token_mint(&self) -> Option<Pubkey> {
        self.token_mint
    }

    fn set_token_mint(&mut self, token_mint: Option<Pubkey>) {
        self.token_mint = token_mint;
    }

    fn get_items_redeemed(&self) -> u64 {
        self.items_redeemed
    }

    fn set_items_redeemed(&mut self, items_redeemed: u64) {
        self.items_redeemed = items_redeemed;
    }

    fn get_price(&self) -> u64 {
        self.data.price
    }

    fn set_price(&mut self, price: u64) {
        self.data.price = price;
    }

    fn get_symbol(&self) -> String {
        self.data.nft_settings.symbol.clone()
    }

    fn set_symbol(&mut self, symbol: String) {
        self.data.nft_settings.symbol = symbol;
    }

    fn get_seller_fee_basis_points(&self) -> u16 {
        self.data.nft_settings.seller_fee_basis_points
    }

    fn set_seller_fee_basis_points(&mut self, seller_fee_basis_points: u16) {
        self.data.nft_settings.seller_fee_basis_points = seller_fee_basis_points;
    }

    fn get_max_supply(&self) -> u64 {
        self.data.nft_settings.max_supply
    }

    fn set_max_supply(&mut self, max_supply: u64) {
        self.data.nft_settings.max_supply = max_supply;
    }

    fn get_is_mutable(&self) -> bool {
        self.data.nft_settings.is_mutable
    }

    fn set_is_mutable(&mut self, is_mutable: bool) {
        self.data.nft_settings.is_mutable = is_mutable;
    }

    fn get_retain_authority(&self) -> bool {
        self.data.nft_settings.retain_authority
    }

    fn set_retain_authority(&mut self, retain_authority: bool) {
        self.data.nft_settings.retain_authority = retain_authority;
    }

    fn get_go_live_date(&self) -> Option<i64> {
        self.data.go_live_date
    }

    fn set_go_live_date(&mut self, go_live_date: Option<i64>) {
        self.data.go_live_date = go_live_date;
    }

    fn get_end_settings(&self) -> Option<EndSettings> {
        self.data.candy_options.end_settings.clone()
    }

    fn set_end_settings(&mut self, end_settings: Option<EndSettings>) {
        self.data.candy_options.end_settings = end_settings;
    }

    fn set_creators(&self) -> Vec<Creator> {
        self.data.nft_settings.creators.clone()
    }

    fn get_creators(&mut self, creators: Vec<Creator>) {
        self.data.nft_settings.creators = creators;
    }

    fn get_hidden_settings(&self) -> Option<HiddenSettings> {
        self.data.candy_options.hidden_settings.clone()
    }

    fn set_hidden_settings(&mut self, hidden_settings: Option<HiddenSettings>) {
        self.data.candy_options.hidden_settings = hidden_settings;
    }

    fn get_whitelist_mint_settings(&self) -> Option<WhitelistMintSettings> {
        self.data.candy_options.whitelist_mint_settings.clone()
    }

    fn set_whitelist_mint_settings(
        &mut self,
        whitelist_mint_settings: Option<WhitelistMintSettings>,
    ) {
        self.data.candy_options.whitelist_mint_settings = whitelist_mint_settings;
    }

    fn get_items_available(&self) -> u64 {
        self.data.items_available
    }

    fn set_items_available(&mut self, items_available: u64) {
        self.data.items_available = items_available;
    }

    fn get_gatekeeper(&self) -> Option<GatekeeperConfig> {
        self.data.candy_options.gatekeeper.clone()
    }

    fn set_gatekeeper(&mut self, gatekeeper: Option<GatekeeperConfig>) {
        self.data.candy_options.gatekeeper = gatekeeper;
    }

    fn get_collection_mint(&self) -> Option<Pubkey> {
        self.data.nft_settings.collection
    }

    fn get_config_array_start(&self) -> usize {
        CONFIG_ARRAY_START_V2
    }
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
    1 + 32 + 1; // gatekeeper

pub const CONFIG_ARRAY_START_V2: usize = 8 + // key
    32 + // authority
    32 + //wallet
    1 + 32 + // token mint
    8 + // items redeemed
    // data
    8 + // price
    8 + // items available
    1 + 8 + // go live
    // candy options
    1 + // WL mint settings
    1 + // mode
    32 + // pubkey
    1 + // presale
    1 + 32 + // discount_price
    1 + // gatekeeper
    32 + 1 +
    1 + // end settings
    1 + 8 +
    1 + // hidden setting
    4 + MAX_NAME_LENGTH + // name length,
    4 + MAX_URI_LENGTH + // uri length,
    32 + // hash
    // nft settings
    8 + // max supply
    4 + MAX_SYMBOL_LENGTH + // u32 len + symbol
    2 + // seller fee basis points
    4 + MAX_CREATOR_LIMIT * MAX_CREATOR_LEN + // creators
    1 + 32 + // collection
    1 + // retain authority
    1 // is mutable
    + 1 // config line settings
    + 1000; // buffer
            // 1749
