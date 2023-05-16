pub mod leaf_schema;
pub mod metadata_model;
pub mod metaplex_adapter;
pub mod metaplex_anchor;

use anchor_lang::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};
use leaf_schema::LeafSchema;

pub const TREE_AUTHORITY_SIZE: usize = 32 + 32 + 8 + 8 + 1 + 15; // 15 bytes padding
pub const VOUCHER_SIZE: usize = 8 + 1 + 32 + 32 + 32 + 8 + 32 + 32 + 4 + 32;
pub const VOUCHER_PREFIX: &str = "voucher";
pub const ASSET_PREFIX: &str = "asset";
pub const COLLECTION_CPI_PREFIX: &str = "collection_cpi";

#[account]
#[derive(Copy, Debug, PartialEq, Eq)]
pub struct TreeConfig {
    pub tree_creator: Pubkey,
    pub tree_delegate: Pubkey,
    pub total_mint_capacity: u64,
    pub num_minted: u64,
    pub is_public: bool,
}

impl TreeConfig {
    pub fn increment_mint_count(&mut self) {
        self.num_minted = self.num_minted.saturating_add(1);
    }

    pub fn contains_mint_capacity(&self, requested_capacity: u64) -> bool {
        let remaining_mints = self.total_mint_capacity.saturating_sub(self.num_minted);
        requested_capacity <= remaining_mints
    }

    pub fn get_metadata_auth_for_v0(&self) -> Pubkey {
        if !self.is_public {
            self.tree_creator.clone()
        } else {
            Pubkey::default()
        }
    }
}

#[account]
#[derive(Debug, Eq, PartialEq)]
pub struct Voucher {
    pub leaf_schema: LeafSchema,
    pub index: u32,
    pub merkle_tree: Pubkey,
}

impl Voucher {
    pub fn new(leaf_schema: LeafSchema, index: u32, merkle_tree: Pubkey) -> Self {
        Self {
            leaf_schema,
            index,
            merkle_tree,
        }
    }

    fn pda_for_prefix(&self, prefix: &str) -> Pubkey {
        Pubkey::find_program_address(
            &[
                prefix.as_ref(),
                self.merkle_tree.as_ref(),
                self.leaf_schema.nonce().to_le_bytes().as_ref(),
            ],
            &crate::id(),
        )
        .0
    }

    pub fn pda(&self) -> Pubkey {
        self.pda_for_prefix(VOUCHER_PREFIX)
    }

    pub fn decompress_mint_pda(&self) -> Pubkey {
        self.pda_for_prefix(ASSET_PREFIX)
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[repr(u8)]
pub enum BubblegumEventType {
    /// Marker for 0 data.
    Uninitialized,
    /// Leaf schema event.
    LeafSchemaEvent,
}
