use anchor_lang::prelude::*;

use super::candy_machine_data::CandyMachineData;

// Indicates the position for the pNFT feature flag. If the bit
// on this position is set to 1, then the candy machine will mint
// pNFTs.
pub const PNFT_FEATURE: u64 = 1 << (std::mem::size_of::<u64>() - 1);

/// Candy machine state and config data.
#[account]
#[derive(Default, Debug)]
pub struct CandyMachine {
    /// Features versioning flags.
    pub features: u64,
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
}

impl CandyMachine {
    pub fn is_enabled(&self, feature: u64) -> bool {
        (self.features & feature) > 0
    }

    pub fn enable_feature(&mut self, feature: u64) {
        self.features |= feature
    }

    pub fn disable_feature(&mut self, feature: u64) {
        self.features &= !feature
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
