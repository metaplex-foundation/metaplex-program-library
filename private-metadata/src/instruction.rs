
use {
    crate::{
        state::{
            MAX_URI_LENGTH,
            CIPHER_KEY_CHUNKS,
        },
        transfer_proof::TransferData,
    },
    bytemuck::{Pod, Zeroable},
    num_derive::{FromPrimitive, ToPrimitive},
    num_traits::{FromPrimitive},
    solana_program::{
        program_error::ProgramError,
        pubkey::Pubkey,
    },
    crate::{
        zk_token_elgamal,
    },
};

#[cfg(not(target_arch = "bpf"))]
use {
    num_traits::{ToPrimitive},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        sysvar,
    },
};

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct ConfigureMetadataData {
    pub elgamal_pk: zk_token_elgamal::pod::ElGamalPubkey,
    pub encrypted_cipher_key: [zk_token_elgamal::pod::ElGamalCiphertext; CIPHER_KEY_CHUNKS],
    pub uri: crate::state::URI,
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct TransferChunkData {
    pub chunk_idx: u8,
    pub transfer: TransferData,
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

pub fn get_metadata_address(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            spl_token_metadata::state::PREFIX.as_bytes(),
            spl_token_metadata::ID.as_ref(),
            mint.as_ref(),
        ],
        &spl_token_metadata::ID,
    )
}

pub fn get_private_metadata_address(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            crate::state::PREFIX.as_bytes(),
            mint.as_ref(),
        ],
        &crate::ID,
    )
}

#[cfg(not(target_arch = "bpf"))]
pub fn encode_instruction<T: Pod>(
    accounts: Vec<AccountMeta>,
    instruction_type: PrivateMetadataInstruction,
    instruction_data: &T,
) -> Instruction {
    let mut data = vec![ToPrimitive::to_u8(&instruction_type).unwrap()];
    data.extend_from_slice(bytemuck::bytes_of(instruction_data));
    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}

#[cfg(not(target_arch = "bpf"))]
pub fn configure_metadata(
    payer: Pubkey,
    mint: Pubkey,
    data: ConfigureMetadataData,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(mint, false),
        AccountMeta::new_readonly(get_metadata_address(&mint).0, false),
        AccountMeta::new_readonly(payer, true),
        AccountMeta::new(get_private_metadata_address(&mint).0, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    encode_instruction(
        accounts,
        PrivateMetadataInstruction::ConfigureMetadata,
        &data,
    )
}

#[cfg(not(target_arch = "bpf"))]
pub fn init_transfer(
    payer: Pubkey,
    mint: Pubkey,
    transfer_buffer: Pubkey,
    elgamal_pk: zk_token_elgamal::pod::ElGamalPubkey,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(mint, false),
        AccountMeta::new_readonly(
            spl_associated_token_account::get_associated_token_address(&payer, &mint),
            false,
        ),
        AccountMeta::new_readonly(get_private_metadata_address(&mint).0, false),
        AccountMeta::new(transfer_buffer, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    encode_instruction(
        accounts,
        PrivateMetadataInstruction::InitTransfer,
        &elgamal_pk,
    )
}

#[cfg(not(target_arch = "bpf"))]
pub fn transfer_chunk(
    payer: Pubkey,
    mint: Pubkey,
    transfer_buffer: Pubkey,
    data: TransferChunkData,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(get_private_metadata_address(&mint).0, false),
        AccountMeta::new(transfer_buffer, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    encode_instruction(
        accounts,
        PrivateMetadataInstruction::TransferChunk,
        &data,
    )
}
