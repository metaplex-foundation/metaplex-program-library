
use crate::state::{
    MAX_URI_LENGTH,
    CIPHER_KEY_CHUNKS,
};
use bytemuck::{Pod, Zeroable};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use solana_program::{
    program_error::ProgramError,
};

use spl_zk_token_sdk::{
    zk_token_elgamal,
};

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct ConfigureMetadataData {
    pub elgamal_pk: zk_token_elgamal::pod::ElGamalPubkey,
    pub encrypted_cipher_key: [zk_token_elgamal::pod::ElGamalCiphertext; CIPHER_KEY_CHUNKS],
    pub uri: [u8; MAX_URI_LENGTH],
}

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum PrivateMetadataInstruction {

    ConfigureMetadata,

    InitTransfer,

    FiniTransfer,

    TransferChunk,
}

pub fn decode_instruction_type(
    input: &[u8]
) -> Result<PrivateMetadataInstruction, ProgramError> {
    if input.is_empty() {
        Err(ProgramError::InvalidInstructionData)
    } else {
        FromPrimitive::from_u8(input[0]).ok_or(ProgramError::InvalidInstructionData)
    }
}

pub fn decode_instruction_data<T: Pod>(
    input: &[u8]
) -> Result<&T, ProgramError> {
    if input.len() < 2 {
        Err(ProgramError::InvalidInstructionData)
    } else {
        pod_from_bytes(&input[1..]).ok_or(ProgramError::InvalidArgument)
    }
}

/// Convert a slice into a `Pod` (zero copy)
pub fn pod_from_bytes<T: Pod>(bytes: &[u8]) -> Option<&T> {
    bytemuck::try_from_bytes(bytes).ok()
}

