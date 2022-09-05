pub mod leaf_schema;
pub mod metaplex_adapter;
pub mod metaplex_anchor;

use anchor_lang::prelude::*;
use leaf_schema::LeafSchema;
use leaf_schema::Version;
use metaplex_adapter::MetadataArgs;

pub const TREE_AUTHORITY_SIZE: usize = 88 + 8;
pub const VOUCHER_SIZE: usize = 8 + 1 + 32 + 32 + 32 + 8 + 32 + 32 + 4 + 32;
pub const VOUCHER_PREFIX: &str = "voucher";
pub const ASSET_PREFIX: &str = "asset";
pub const COLLECTION_CPI_PREFIX: &str = "collection_cpi";

#[account]
#[derive(Copy, Debug)]
pub struct TreeConfig {
    pub tree_creator: Pubkey,
    pub tree_delegate: Pubkey,
    pub total_mint_capacity: u64,
    pub num_minted: u64,
}

impl TreeConfig {
    pub fn increment_mint_count(&mut self) {
        self.num_minted = self.num_minted.saturating_add(1);
    }

    pub fn contains_mint_capacity(&self, requested_capacity: u64) -> bool {
        let remaining_mints = self.total_mint_capacity.saturating_sub(self.num_minted);
        requested_capacity <= remaining_mints
    }
}

#[account]
#[derive(Copy)]
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
