use {
    crate::{pod::*},
    bytemuck::{Pod, Zeroable},
    solana_program::pubkey::Pubkey,
    spl_zk_token_sdk::zk_token_elgamal::{self},
};

pub const PREFIX: &str = "metadata";

// TODO: whats actually the max we can reasonably encrypt?
pub const MAX_URI_LENGTH: usize = 30;
pub const MAX_METADATA_LEN: usize
    = 8                     // discriminator
    + 32                    // mint
    + 32                    // elgamal pubkey
    + MAX_URI_LENGTH        // uri
    ;

/// Account data
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct PrivateMetadataAccount {
    /// The corresponding SPL Token Mint
    pub mint: Pubkey,

    /// The public key associated with ElGamal encryption
    pub elgamal_pk: zk_token_elgamal::pod::ElGamalPubkey,

    /// URI of encrypted asset
    pub uri: [u8; MAX_URI_LENGTH],
}
impl PodAccountInfo<'_, '_> for PrivateMetadataAccount {}

