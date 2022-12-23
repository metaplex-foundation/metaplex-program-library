use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

use super::{Collection, CollectionDetails, Creator, Data, DataV2, TokenStandard, Uses};

/// Data representation of an asset.
#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct AssetData {
    /// Update Authority for the asset.
    pub update_authority: Pubkey,
    /// The name of the asset.
    pub name: String,
    /// The symbol for the asset.
    pub symbol: String,
    /// URI pointing to JSON representing the asset.
    pub uri: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000).
    pub seller_fee_basis_points: u16,
    /// Array of creators.
    pub creators: Option<Vec<Creator>>,
    // Immutable, once flipped, all sales of this metadata are considered secondary.
    pub primary_sale_happened: bool,
    // Whether or not the data struct is mutable (default is not).
    pub is_mutable: bool,
    /// nonce for easy calculation of editions (if present).
    pub edition_nonce: Option<u8>,
    /// Type of the token.
    pub token_standard: TokenStandard,
    /// Collection information.
    pub collection: Option<Collection>,
    /// Uses information.
    pub uses: Option<Uses>,
    /// Collection item details.
    pub collection_details: Option<CollectionDetails>,
    // Programmable configuration for the asset.
    pub programmable_config: Option<ProgrammableConfig>,
    // Delegate status
    pub delegate_state: Option<DelegateState>,
}

impl AssetData {
    pub fn new(
        token_standard: TokenStandard,
        name: String,
        symbol: String,
        uri: String,
        update_authority: Pubkey,
    ) -> Self {
        Self {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            update_authority,
            creators: None,
            primary_sale_happened: false,
            is_mutable: true,
            edition_nonce: None,
            token_standard,
            collection: None,
            uses: None,
            collection_details: None,
            programmable_config: None,
            delegate_state: None,
        }
    }

    pub fn as_data_v2(&self) -> DataV2 {
        DataV2 {
            collection: self.collection.clone(),
            creators: self.creators.clone(),
            name: self.name.clone(),
            seller_fee_basis_points: self.seller_fee_basis_points,
            symbol: self.symbol.clone(),
            uri: self.uri.clone(),
            uses: self.uses.clone(),
        }
    }

    pub fn as_data(&self) -> Data {
        Data {
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            uri: self.uri.clone(),
            seller_fee_basis_points: self.seller_fee_basis_points,
            creators: self.creators.clone(),
        }
    }
}

/// Configuration of the programmable rules.
#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct ProgrammableConfig {
    pub rule_set: Pubkey,
}

/// Different delegate states for a token. Some actions are
/// not allowed depending of the degate state.
#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum DelegateState {
    Transfer(Pubkey),
}
