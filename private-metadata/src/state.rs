use {
    crate::{pod::*},
    bytemuck::{Pod, Zeroable},
    solana_program::pubkey::Pubkey,
    spl_zk_token_sdk::zk_token_elgamal,
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

pub const CIPHER_KEY_CHUNKS: usize = 6;

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Key {
    Uninitialized,
    PrivateMetadataAccountV1,
    CipherKeyTransferBufferV1,
}

// wcgw
unsafe impl Zeroable for Key {}
unsafe impl Pod for Key {}

/// Account data
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct PrivateMetadataAccount {
    pub key: Key,

    /// The corresponding SPL Token Mint
    pub mint: Pubkey,

    /// The public key associated with ElGamal encryption
    pub elgamal_pk: zk_token_elgamal::pod::ElGamalPubkey,

    /// 192-bit AES cipher key encrypted with elgamal_pk
    /// ElGamalCiphertext encrypted 4-byte chunks so 6 chunks total
    pub encrypted_cipher_key: [zk_token_elgamal::pod::ElGamalCiphertext; CIPHER_KEY_CHUNKS],

    /// URI of encrypted asset
    pub uri: [u8; MAX_URI_LENGTH],
}
impl PodAccountInfo<'_, '_> for PrivateMetadataAccount {}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct CipherKeyTransferBuffer {
    pub key: Key,

    /// Bit mask of updated chunks
    pub updated: u8,

    /// Source pubkey. Should match the currently encrypted elgamal_pk
    pub authority: Pubkey,

    /// Account that will have its encrypted key updated
    pub private_metadata_key: Pubkey,

    /// Destination public key
    pub elgamal_pk: zk_token_elgamal::pod::ElGamalPubkey,

    /// 192-bit AES cipher key encrypted with elgamal_pk
    pub encrypted_cipher_key: [zk_token_elgamal::pod::ElGamalCiphertext; CIPHER_KEY_CHUNKS],
}
impl PodAccountInfo<'_, '_> for CipherKeyTransferBuffer {}
