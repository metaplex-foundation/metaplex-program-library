use anchor_lang::prelude::*;

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
