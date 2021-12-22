
use {
    crate::{
        state::{
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
    crate::equality_proof,
    num_traits::{ToPrimitive},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        sysvar,
        system_instruction,
    },
    solana_sdk::signer::Signer,
    std::convert::TryInto,
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

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct TransferChunkSlowData {
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

    TransferChunkSlow,
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
    elgamal_pk: zk_token_elgamal::pod::ElGamalPubkey,
    encrypted_cipher_key: &[zk_token_elgamal::pod::ElGamalCiphertext],
    uri: &[u8],
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(mint, false),
        AccountMeta::new_readonly(get_metadata_address(&mint).0, false),
        AccountMeta::new_readonly(payer, true),
        AccountMeta::new(get_private_metadata_address(&mint).0, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    let mut data = ConfigureMetadataData::zeroed();
    data.elgamal_pk = elgamal_pk;
    data.encrypted_cipher_key.copy_from_slice(encrypted_cipher_key);
    data.uri.0[..uri.len()].copy_from_slice(uri);

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
    let accounts = vec![
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
    let accounts = vec![
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

#[cfg(not(target_arch = "bpf"))]
pub fn transfer_chunk_slow(
    payer: Pubkey,
    mint: Pubkey,
    transfer_buffer: Pubkey,
    instruction_buffer: Pubkey,
    input_buffer: Pubkey,
    compute_buffer: Pubkey,
    data: TransferChunkSlowData,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(get_private_metadata_address(&mint).0, false),
        AccountMeta::new(transfer_buffer, false),
        AccountMeta::new_readonly(instruction_buffer, false),
        AccountMeta::new_readonly(input_buffer, false),
        AccountMeta::new_readonly(compute_buffer, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    encode_instruction(
        accounts,
        PrivateMetadataInstruction::TransferChunkSlow,
        &data,
    )
}

#[cfg(not(target_arch = "bpf"))]
pub struct InstructionsAndSigners<'a> {
    pub instructions: Vec<Instruction>,
    pub signers: Vec<&'a dyn Signer>,
}

#[cfg(not(target_arch = "bpf"))]
pub fn populate_transfer_proof_dsl<'a, F>(
    payer: &'a dyn Signer,
    instruction_buffer: &'a dyn Signer,
    minimum_rent_balance: F,
) -> Vec<InstructionsAndSigners<'a>>
    where F: Fn(usize) -> u64,
{
    use curve25519_dalek_onchain::instruction as dalek;

    let dsl_len = equality_proof::DSL_INSTRUCTION_BYTES.len();
    let instruction_buffer_len = dalek::HEADER_SIZE + dsl_len;

    let mut ret = vec![];

    ret.push(InstructionsAndSigners{
        instructions: vec![
            system_instruction::create_account(
                &payer.pubkey(),
                &instruction_buffer.pubkey(),
                minimum_rent_balance(instruction_buffer_len),
                instruction_buffer_len as u64,
                &curve25519_dalek_onchain::id(),
            ),
            dalek::initialize_buffer(
                instruction_buffer.pubkey(),
                payer.pubkey(),
                dalek::Key::InstructionBufferV1,
                vec![],
            ),
        ],
        signers: vec![payer, instruction_buffer],
    });

    // write the instructions
    let mut dsl_idx = 0;
    let dsl_chunk = 800;
    loop {
        let mut instructions = vec![];
        let end = (dsl_idx+dsl_chunk).min(dsl_len);
        let done = end == dsl_len;
        instructions.push(
            dalek::write_bytes(
                instruction_buffer.pubkey(),
                payer.pubkey(),
                (dalek::HEADER_SIZE + dsl_idx).try_into().unwrap(),
                done,
                &equality_proof::DSL_INSTRUCTION_BYTES[dsl_idx..end],
            )
        );
        ret.push(InstructionsAndSigners{
            instructions,
            signers: vec![payer],
        });
        if done {
            break;
        } else {
            dsl_idx = end;
        }
    }

    ret
}

// Returns a list of transaction instructions that can be sent to build the zk proof state used in
// a `transfer_chunk_slow`. These instructions assume that the instruction DSL has already been
// populated with `populate_transfer_proof_dsl`
#[cfg(not(target_arch = "bpf"))]
pub fn transfer_chunk_slow_proof<'a, F>(
    payer: &'a dyn Signer,
    instruction_buffer: Pubkey,
    input_buffer: &'a dyn Signer,
    compute_buffer: &'a dyn Signer,
    transfer: &'a TransferData,
    minimum_rent_balance: F,
) -> Vec<InstructionsAndSigners<'a>>
    where F: Fn(usize) -> u64,
{
    use crate::transcript::TranscriptProtocol;
    use crate::transfer_proof::TransferProof;
    use curve25519_dalek::scalar::Scalar;
    use curve25519_dalek_onchain::instruction as dalek;
    use curve25519_dalek_onchain::scalar::Scalar as OScalar;

    let equality_proof = equality_proof::EqualityProof::from_bytes(
        &transfer.proof.equality_proof.0).unwrap();

    let points = [
        // statement inputs
        transfer.transfer_public_keys.src_pubkey.0,
        equality_proof::COMPRESSED_H,
        equality_proof.Y_0.0,

        transfer.transfer_public_keys.dst_pubkey.0,
        transfer.dst_cipher_key_chunk_ct.0[32..].try_into().unwrap(),
        equality_proof.Y_1.0,

        transfer.dst_cipher_key_chunk_ct.0[..32].try_into().unwrap(),
        transfer.src_cipher_key_chunk_ct.0[..32].try_into().unwrap(),
        transfer.src_cipher_key_chunk_ct.0[32..].try_into().unwrap(),
        equality_proof::COMPRESSED_H,
        equality_proof.Y_2.0,
    ];

    let mut transcript = TransferProof::transcript_new();
    TransferProof::build_transcript(
        &transfer.src_cipher_key_chunk_ct,
        &transfer.dst_cipher_key_chunk_ct,
        &transfer.transfer_public_keys,
        &mut transcript,
    ).unwrap();

    equality_proof::EqualityProof::build_transcript(
        &equality_proof,
        &mut transcript,
    ).unwrap();

    let challenge_c = transcript.challenge_scalar(b"c");

    // the equality_proof points are normal 'Scalar' but the DSL crank expects it's version of the
    // type
    let scalars = vec![
         equality_proof.sh_1,
         -challenge_c,
         -Scalar::one(),

         equality_proof.rh_2,
         -challenge_c,
         -Scalar::one(),

         challenge_c,
         -challenge_c,
         equality_proof.sh_1,
         -equality_proof.rh_2,
         -Scalar::one(),
    ]
        .iter()
        .map(|s| OScalar::from_canonical_bytes(s.bytes).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(points.len(), scalars.len());

    let input_buffer_len = dalek::HEADER_SIZE + points.len() * 32 * 2 + 128;

    let compute_buffer_len =
        dalek::HEADER_SIZE
        + 3 * 32 * 4                 // 3 proof groups
        + 32 * 12                    // decompression space
        + 32 * scalars.len()         // scalars
        + 32 * 4 * 8 * points.len()  // point lookup tables
        ;

    let mut ret = vec![];

    ret.push(InstructionsAndSigners{
        instructions: vec![
            system_instruction::create_account(
                &payer.pubkey(),
                &input_buffer.pubkey(),
                minimum_rent_balance(input_buffer_len),
                input_buffer_len as u64,
                &curve25519_dalek_onchain::id(),
            ),
            system_instruction::create_account(
                &payer.pubkey(),
                &compute_buffer.pubkey(),
                minimum_rent_balance(compute_buffer_len),
                compute_buffer_len as u64,
                &curve25519_dalek_onchain::id(),
            ),
            dalek::initialize_buffer(
                input_buffer.pubkey(),
                payer.pubkey(),
                dalek::Key::InputBufferV1,
                vec![],
            ),
            dalek::initialize_buffer(
                compute_buffer.pubkey(),
                payer.pubkey(),
                dalek::Key::ComputeBufferV1,
                vec![instruction_buffer, input_buffer.pubkey()],
            ),
        ],
        signers: vec![payer, input_buffer, compute_buffer],
    });

    ret.push(InstructionsAndSigners{
        instructions: dalek::write_input_buffer(
            input_buffer.pubkey(),
            payer.pubkey(),
            &points,
            scalars.as_slice(),
        ),
        signers: vec![payer],
    });

    let instructions_per_tx = 32;
    let num_cranks = equality_proof::DSL_INSTRUCTION_COUNT;
    let mut current = 0;
    while current < num_cranks {
        let mut instructions = vec![];
        for _j in 0..instructions_per_tx {
            if current >= num_cranks {
                break;
            }
            instructions.push(
                dalek::crank_compute(
                    instruction_buffer,
                    input_buffer.pubkey(),
                    compute_buffer.pubkey(),
                ),
            );
            current += 1;
        }
        ret.push(InstructionsAndSigners{
            instructions,
            signers: vec![payer],
        });
    }

    ret
}
