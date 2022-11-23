use crate::{constants::FREEZE_FEE, CandyError};
use anchor_lang::prelude::*;

/// Candy machine state and config data.
#[account]
#[derive(Default, Debug)]
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

/// Collection PDA account
#[account]
#[derive(Default, Debug)]
pub struct CollectionPDA {
    pub mint: Pubkey,
    pub candy_machine: Pubkey,
}

impl CollectionPDA {
    pub const PREFIX: &'static str = "collection";
}

/// Collection PDA account
#[account]
#[derive(Default, Debug, PartialEq, Eq)]
pub struct FreezePDA {
    // duplicate key in order to find the candy machine without txn crawling
    pub candy_machine: Pubkey,   // 32
    pub allow_thaw: bool,        // 1
    pub frozen_count: u64,       // 8
    pub mint_start: Option<i64>, // 1 + 8
    pub freeze_time: i64,        // 8
    pub freeze_fee: u64,         // 8
}

impl FreezePDA {
    pub const SIZE: usize = 8 + 32 + 32 + 1 + 8 + 1 + 8 + 8 + 8;

    pub const PREFIX: &'static str = "freeze";

    pub fn init(&mut self, candy_machine: Pubkey, mint_start: Option<i64>, freeze_time: i64) {
        self.candy_machine = candy_machine;
        self.allow_thaw = false;
        self.frozen_count = 0;
        self.mint_start = mint_start;
        self.freeze_time = freeze_time;
        self.freeze_fee = FREEZE_FEE;
    }

    pub fn thaw_eligible(&self, current_timestamp: i64, candy_machine: &CandyMachine) -> bool {
        if self.allow_thaw || candy_machine.items_redeemed >= candy_machine.data.items_available {
            return true;
        } else if let Some(start_timestamp) = self.mint_start {
            if current_timestamp >= start_timestamp + self.freeze_time {
                return true;
            }
        }
        false
    }

    pub fn assert_from_candy(&self, candy_machine: &Pubkey) -> Result<()> {
        if &self.candy_machine != candy_machine {
            return err!(CandyError::FreezePDAMismatch);
        }
        Ok(())
    }
}

/// Candy machine settings data.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Debug)]
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

impl CandyMachine {
    pub fn assert_not_minted(&self, candy_error: Error) -> Result<()> {
        if self.items_redeemed > 0 {
            Err(candy_error)
        } else {
            Ok(())
        }
    }
}

/// Individual config line for storing NFT data pre-mint.
#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ConfigLine {
    pub name: String,
    /// URI pointing to JSON representing the asset
    pub uri: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EndSettings {
    pub end_setting_type: EndSettingType,
    pub number: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum EndSettingType {
    Date,
    Amount,
}

// Unfortunate duplication of token metadata so that IDL picks it up.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    // In percentages, NOT basis points ;) Watch out!
    pub share: u8,
}

/// Hidden Settings for large mints used with offline data.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Debug)]
pub struct HiddenSettings {
    pub name: String,
    pub uri: String,
    pub hash: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct WhitelistMintSettings {
    pub mode: WhitelistMintMode,
    pub mint: Pubkey,
    pub presale: bool,
    pub discount_price: Option<u64>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Eq, PartialEq, Debug)]
pub enum WhitelistMintMode {
    // Only captcha uses the bytes, the others just need to have same length
    // for front end borsh to not crap itself
    // Holds the validation window
    BurnEveryTime,
    NeverBurn,
}

/// Configurations options for the gatekeeper.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct GatekeeperConfig {
    /// The network for the gateway token required
    pub gatekeeper_network: Pubkey,
    /// Whether or not the token should expire after minting.
    /// The gatekeeper network must support this if true.
    pub expire_on_use: bool,
}
