//! Module provide program defined state

use crate::utils::{DESCRIPTION_DEFAULT_SIZE, MAX_PRIMARY_CREATORS_LEN, NAME_DEFAULT_SIZE};
use anchor_lang::prelude::*;
use mpl_token_metadata::state::Creator as MPL_Creator;
use std::convert::From;

// by system acc I mean account to hold only native SOL
pub const MINIMUM_BALANCE_FOR_SYSTEM_ACCS: u64 = 890880;

#[account]
pub struct Store {
    pub admin: Pubkey,
    pub name: String,
    pub description: String,
}

impl Store {
    pub const LEN: usize = 8 + 32 + NAME_DEFAULT_SIZE + DESCRIPTION_DEFAULT_SIZE;
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Debug, PartialEq, Eq)]
pub enum SellingResourceState {
    Uninitialized,
    Created,
    InUse,
    Exhausted,
    Stopped,
}

#[account]
pub struct SellingResource {
    pub store: Pubkey,
    pub owner: Pubkey,
    pub resource: Pubkey,
    pub vault: Pubkey,
    pub vault_owner: Pubkey,
    pub supply: u64,
    pub max_supply: Option<u64>,
    pub state: SellingResourceState,
}

impl SellingResource {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 32 + 32 + 8 + 9 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum MarketState {
    Uninitialized,
    Created,
    Suspended,
    Active,
    Ended,
}

#[account]
pub struct Market {
    pub store: Pubkey,
    pub selling_resource: Pubkey,
    pub treasury_mint: Pubkey,
    pub treasury_holder: Pubkey,
    pub treasury_owner: Pubkey,
    pub owner: Pubkey,
    pub name: String,
    pub description: String,
    pub mutable: bool,
    pub price: u64,
    pub pieces_in_one_wallet: Option<u64>,
    pub start_date: u64,
    pub end_date: Option<u64>,
    pub state: MarketState,
    // need this field to calculate royalties at withdraw
    pub funds_collected: u64,
    pub gatekeeper: Option<GatingConfig>,
}

impl Market {
    pub const LEN: usize = 8
        + 32
        + 32
        + 32
        + 32
        + 32
        + 32
        + NAME_DEFAULT_SIZE
        + DESCRIPTION_DEFAULT_SIZE
        + 1
        + 8
        + 9
        + 8
        + 9
        + 1
        + 8
        + 1
        + 32
        + 1
        + 9;
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Debug, PartialEq, Eq)]
pub struct GatingConfig {
    pub collection: Pubkey,
    /// whether program will burn token or just check availability
    pub expire_on_use: bool,
    pub gating_time: Option<u64>,
}

#[account]
#[derive(Default)]
pub struct TradeHistory {
    pub market: Pubkey,
    pub wallet: Pubkey,
    pub already_bought: u64,
}

impl TradeHistory {
    pub const LEN: usize = 8 + 32 + 32 + 8;
}

#[account]
pub struct PrimaryMetadataCreators {
    pub creators: Vec<Creator>,
}


pub fn from_mpl_creators(creators: Vec<mpl_token_metadata::state::Creator>) -> Vec<Creator> {
    creators
        .iter()
        .map(|e| Creator {
            address: e.address,
            share: e.share,
            verified: e.verified,
        })
        .collect()
}

impl PrimaryMetadataCreators {
    pub const LEN: usize = 8 + ((32 + 1 + 1) * MAX_PRIMARY_CREATORS_LEN + 1);
}

#[account]
#[derive(Default)]
pub struct PayoutTicket {
    pub used: bool,
}

impl PayoutTicket {
    pub const LEN: usize = 9;
}

// Unfortunate duplication of token metadata so that IDL picks it up.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    // In percentages, NOT basis points ;) Watch out!
    pub share: u8,
}

impl From<MPL_Creator> for Creator {
    fn from(item: MPL_Creator) -> Self {
        Creator {
            address: item.address,
            verified: item.verified,
            share: item.share,
        }
    }
}
