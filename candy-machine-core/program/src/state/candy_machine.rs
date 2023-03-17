use anchor_lang::prelude::*;
use arrayref::array_ref;
use mpl_token_metadata::state::{Metadata, ProgrammableConfig};

use crate::constants::{RULE_SET_LENGTH, SET};

use super::candy_machine_data::CandyMachineData;

/// Candy machine state and config data.
#[account]
#[derive(Default, Debug)]
pub struct CandyMachine {
    /// Version of the account.
    pub version: AccountVersion,
    /// Token standard to mint NFTs.
    pub token_standard: u8,
    /// Features flags.
    pub features: [u8; 6],
    /// Authority address.
    pub authority: Pubkey,
    /// Authority address allowed to mint from the candy machine.
    pub mint_authority: Pubkey,
    /// The collection mint for the candy machine.
    pub collection_mint: Pubkey,
    /// Number of assets redeemed.
    pub items_redeemed: u64,
    /// Candy machine configuration data.
    pub data: CandyMachineData,
    // hidden data section to avoid deserialisation:
    //
    // - (u32) how many actual lines of data there are currently (eventually
    //   equals items available)
    // - (ConfigLine * items_available) lines and lines of name + uri data
    // - (item_available / 8) + 1 bit mask to keep track of which ConfigLines
    //   have been added
    // - (u32 * items_available) mint indices
    // - for pNFT:
    //   (u8) indicates whether to use a custom rule set
    //   (Pubkey) custom rule set
}

impl CandyMachine {
    pub fn get_rule_set(
        &self,
        account_data: &[u8],
        collection_metadata: &Metadata,
    ) -> Result<Option<Pubkey>> {
        let required_length = self.data.get_space_for_candy()?;

        if account_data[required_length] == SET {
            let index = required_length + 1;

            Ok(Some(Pubkey::from(*array_ref![
                account_data,
                index,
                RULE_SET_LENGTH
            ])))
        } else if let Some(ProgrammableConfig::V1 { rule_set }) =
            collection_metadata.programmable_config
        {
            Ok(rule_set)
        } else {
            Ok(None)
        }
    }
}

/// Config line struct for storing asset (NFT) data pre-mint.
#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ConfigLine {
    /// Name of the asset.
    pub name: String,
    /// URI to JSON metadata.
    pub uri: String,
}

/// Account versioning.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Debug)]
pub enum AccountVersion {
    #[default]
    V1,
    V2,
}
