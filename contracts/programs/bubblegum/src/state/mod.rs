pub mod leaf_schema;
pub mod metaplex_adapter;
pub mod metaplex_anchor;
pub mod request;

use anchor_lang::prelude::*;
use leaf_schema::LeafSchema;
use leaf_schema::Version;
use metaplex_adapter::MetadataArgs;

pub const TREE_AUTHORITY_SIZE: usize = 88 + 8;
pub const VOUCHER_SIZE: usize = 8 + 1 + 32 + 32 + 32 + 8 + 32 + 32 + 4 + 32;
pub const VOUCHER_PREFIX: &str = "voucher";
pub const ASSET_PREFIX: &str = "asset";
#[account]
#[derive(Copy)]
pub struct TreeConfig {
    pub creator: Pubkey,
    pub delegate: Pubkey,
    pub total_mint_capacity: u64,
    pub num_mints_approved: u64,
    pub num_minted: u64,
}

impl TreeConfig {
    pub fn increment_mint_count(&mut self) {
        self.num_minted = self.num_minted.saturating_add(1);
    }

    pub fn approve_mint_capacity(&mut self, capacity: u64) {
        self.num_mints_approved = self.num_mints_approved.saturating_add(capacity);
    }

    pub fn contains_mint_capacity(&self, requested_capacity: u64) -> bool {
        let remaining_mints_to_approve = self
            .total_mint_capacity
            .saturating_sub(self.num_mints_approved);
        let remaining_mints = self.total_mint_capacity.saturating_sub(self.num_minted);
        requested_capacity <= remaining_mints && requested_capacity <= remaining_mints_to_approve
    }

    pub fn restore_mint_capacity(&mut self, capacity: u64) {
        self.num_mints_approved = self.num_mints_approved.saturating_sub(capacity);
    }
}

#[account]
#[derive(Copy)]
pub struct Voucher {
    pub leaf_schema: LeafSchema,
    pub index: u32,
    pub merkle_slab: Pubkey,
}

impl Voucher {
    pub fn new(leaf_schema: LeafSchema, index: u32, merkle_slab: Pubkey) -> Self {
        Self {
            leaf_schema,
            index,
            merkle_slab,
        }
    }
}

#[event]
pub struct NewNFTEvent {
    pub version: Version,
    pub metadata: MetadataArgs,
    pub nonce: u64,
}

#[event]
pub struct NFTDecompressionEvent {
    pub version: Version,
    pub id: Pubkey,
    pub tree_id: Pubkey,
    pub nonce: u64,
}
