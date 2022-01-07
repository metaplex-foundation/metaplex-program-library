use {
    crate::{
        pod::{PodAccountInfo, PodBool},
        zk_token_elgamal,
    },
    bytemuck::{Pod, Zeroable},
    num_derive::{
        FromPrimitive,
        ToPrimitive,
    },
    solana_program::pubkey::Pubkey,
};

pub const PREFIX: &str = "metadata";

pub const MAX_URI_LENGTH: usize = 100;
pub const MAX_METADATA_LEN: usize
    = 8                     // discriminator
    + 32                    // mint
    + 32                    // elgamal pubkey
    + MAX_URI_LENGTH        // uri
    ;

#[derive(Clone, Copy, PartialEq, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum Key {
    Uninitialized,
    PrivateMetadataAccountV1,
    CipherKeyTransferBufferV1,
}

// wcgw
unsafe impl Zeroable for Key {}
unsafe impl Pod for Key {}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct URI(pub [u8; MAX_URI_LENGTH]);

unsafe impl Zeroable for URI {}
unsafe impl Pod for URI {}

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
    pub encrypted_cipher_key: zk_token_elgamal::pod::ElGamalCiphertext,

    /// TODO: optional auditor pk and cipher key

    /// URI of encrypted asset
    pub uri: URI,
}
impl PodAccountInfo<'_, '_> for PrivateMetadataAccount {}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct CipherKeyTransferBuffer {
    pub key: Key,

    pub updated: PodBool,

    /// Source pubkey. Should match the currently encrypted elgamal_pk
    pub authority: Pubkey,

    /// Account that will have its encrypted key updated
    pub private_metadata_key: Pubkey,

    /// Destination public key
    pub elgamal_pk: zk_token_elgamal::pod::ElGamalPubkey,

    /// 192-bit AES cipher key encrypted with elgamal_pk
    pub encrypted_cipher_key: zk_token_elgamal::pod::ElGamalCiphertext,
}
impl PodAccountInfo<'_, '_> for CipherKeyTransferBuffer {}
