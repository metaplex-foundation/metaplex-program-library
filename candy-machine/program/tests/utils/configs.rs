use solana_program::pubkey::Pubkey;
use solana_sdk::signer::Signer;

use mpl_candy_machine::{
    CandyMachineData, Creator, EndSettings, GatekeeperConfig, HiddenSettings, WhitelistMintSettings,
};

use crate::CandyManager;

pub const DEFAULT_UUID: &str = "ABCDEF";
pub const DEFAULT_PRICE: u64 = 1e9 as u64;
pub const ITEMS_AVAILABLE: u64 = 11;
pub const DEFAULT_SYMBOL: &str = "SYMBOL";

#[allow(dead_code)]
pub fn quick_config(creator: Pubkey) -> CandyMachineData {
    custom_config(creator, None, true, true, None, None, None, None)
}

pub fn auto_config(
    candy_manager: &CandyManager,
    go_live_date: Option<i64>,
    is_mutable: bool,
    retain_authority: bool,
    end_settings: Option<EndSettings>,
    hidden_settings: Option<HiddenSettings>,
) -> CandyMachineData {
    let wl_config = candy_manager.whitelist_info.clone();
    let wl_settings = match candy_manager.whitelist_info.set {
        true => Some(WhitelistMintSettings {
            mode: wl_config.whitelist_config.burn,
            mint: wl_config.mint,
            presale: wl_config.whitelist_config.presale,
            discount_price: wl_config.whitelist_config.discount_price,
        }),
        false => None,
    };
    custom_config(
        candy_manager.authority.pubkey(),
        go_live_date,
        is_mutable,
        retain_authority,
        end_settings,
        hidden_settings,
        wl_settings,
        None,
    )
}

pub fn custom_config(
    creator: Pubkey,
    go_live_date: Option<i64>,
    is_mutable: bool,
    retain_authority: bool,
    end_settings: Option<EndSettings>,
    hidden_settings: Option<HiddenSettings>,
    whitelist_mint_settings: Option<WhitelistMintSettings>,
    gatekeeper: Option<GatekeeperConfig>,
) -> CandyMachineData {
    CandyMachineData {
        uuid: DEFAULT_UUID.to_string(),
        items_available: ITEMS_AVAILABLE,
        price: DEFAULT_PRICE,
        symbol: DEFAULT_SYMBOL.to_string(),
        seller_fee_basis_points: 500,
        max_supply: 0,
        creators: vec![Creator {
            address: creator,
            verified: true,
            share: 100,
        }],
        is_mutable,
        retain_authority,
        go_live_date,
        end_settings,
        hidden_settings,
        whitelist_mint_settings,
        gatekeeper,
    }
}
