
use crate::{
    instruction::*,
    state::*,
    error::*,
    pod::*,
    transfer_proof::{Verifiable, TransferData, TransferProof},
    equality_proof::*,
    transcript::TranscriptProtocol,
    zk_token_elgamal,
    ID,
};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_pack::Pack,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use std::convert::TryInto;

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    use borsh::BorshDeserialize;
    match StealthInstruction::try_from_slice(input)? {
        StealthInstruction::ConfigureMetadata(metadata) => {
            msg!("ConfigureMetadata!");
            process_configure_metadata(
                accounts,
                &metadata,
            )
        }
        StealthInstruction::InitTransfer => {
            msg!("InitTransfer!");
            process_init_transfer(
                accounts,
            )
        }
        StealthInstruction::FiniTransfer => {
            msg!("FiniTransfer!");
            process_fini_transfer(
                accounts,
            )
        }
        StealthInstruction::TransferChunk(data) => {
            msg!("TransferChunk!");
            process_transfer_chunk(
                accounts,
                &data.transfer,
                verify_syscall,
            )
        }
        StealthInstruction::TransferChunkSlow(data) => {
            msg!("TransferChunkSlow!");
            process_transfer_chunk(
                accounts,
                &data.transfer,
                verify_dsl_crank,
            )
        }
        StealthInstruction::PublishElgamalPubkey(key) => {
            msg!("PublishElgamalPubkey!");
            process_publish_elgamal_pubkey(
                accounts,
                &key,
            )
        }
        StealthInstruction::CloseElgamalPubkey => {
            msg!("CloseElgamalPubkey!");
            process_close_elgamal_pubkey(
                accounts,
            )
        }
        StealthInstruction::UpdateMetadata(metadata) => {
            msg!("UpdateMetadata!");
            process_update_metadata(
                accounts,
                &metadata,
            )
        }
    }
}

fn process_configure_metadata(
    accounts: &[AccountInfo],
    data: &ConfigureMetadataData
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let payer_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let metadata_update_authority_info = next_account_info(account_info_iter)?;
    let stealth_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let rent_sysvar_info = next_account_info(account_info_iter)?;

    if !payer_info.is_signer {
        msg!("Payer is not a signer");
        return Err(ProgramError::InvalidArgument);
    }

    if !metadata_update_authority_info.is_signer {
        msg!("Metadata update authority is not a signer");
        return Err(ProgramError::InvalidArgument);
    }
    validate_account_owner(mint_info, &spl_token::ID)?;
    validate_account_owner(metadata_info, &mpl_token_metadata::ID)?;

    let metadata = validate_metadata(metadata_info, mint_info)?;
    if metadata.update_authority != *metadata_update_authority_info.key {
        msg!("Invalid metadata update authority");
        return Err(StealthError::InvalidUpdateAuthority.into());
    }


    // check that Stealth matches mint
    let (stealth_key, stealth_bump_seed) =
        get_stealth_address(mint_info.key);

    if stealth_key != *stealth_info.key {
        msg!("Invalid stealth key");
        return Err(StealthError::InvalidStealthKey.into());
    }

    // create and initialize PDA
    mpl_token_metadata::utils::create_or_allocate_account_raw(
        ID,
        stealth_info,
        rent_sysvar_info,
        system_program_info,
        payer_info,
        StealthAccount::get_packed_len(),
        &[
            PREFIX.as_bytes(),
            mint_info.key.as_ref(),
            &[stealth_bump_seed],
        ],
    )?;

    let mut stealth = StealthAccount::from_account_info(
        &stealth_info, &ID, Key::Uninitialized)?.into_mut();

    stealth.key = Key::StealthAccountV1;
    stealth.mint = *mint_info.key;
    stealth.wallet_pk = *payer_info.key;
    stealth.elgamal_pk = data.elgamal_pk;
    stealth.encrypted_cipher_key = data.encrypted_cipher_key;
    stealth.uri = data.uri;
    stealth.bump_seed = stealth_bump_seed;

    Ok(())
}

fn process_update_metadata(
    accounts: &[AccountInfo],
    data: &ConfigureMetadataData
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let payer_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let owner_token_account_info = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let metadata_update_authority_info = next_account_info(account_info_iter)?;
    let stealth_info = next_account_info(account_info_iter)?;

    if !payer_info.is_signer {
        msg!("Payer is not a signer");
        return Err(ProgramError::InvalidArgument);
    }

    if !metadata_update_authority_info.is_signer {
        msg!("Metadata update authority is not a signer");
        return Err(ProgramError::InvalidArgument);
    }
    validate_account_owner(mint_info, &spl_token::ID)?;
    validate_account_owner(owner_token_account_info, &spl_token::ID)?;
    validate_account_owner(metadata_info, &mpl_token_metadata::ID)?;
    validate_account_owner(stealth_info, &ID)?;

    let metadata = validate_metadata(metadata_info, mint_info)?;
    if metadata.update_authority != *metadata_update_authority_info.key {
        msg!("Invalid metadata update authority");
        return Err(StealthError::InvalidUpdateAuthority.into());
    }


    // check against owner
    let owner_token_account = spl_token::state::Account::unpack(
        &owner_token_account_info.data.borrow())?;

    if owner_token_account.mint != *mint_info.key {
        msg!("Mint mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if owner_token_account.owner != *owner_info.key {
        msg!("Owner mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if owner_token_account.amount != 1 {
        msg!("Invalid amount");
        return Err(ProgramError::InvalidArgument);
    }


    // check that Stealth matches mint
    let (stealth_key, _stealth_bump_seed) =
        get_stealth_address(mint_info.key);

    if stealth_key != *stealth_info.key {
        msg!("Invalid stealth key");
        return Err(StealthError::InvalidStealthKey.into());
    }

    let mut stealth = StealthAccount::from_account_info(
        &stealth_info, &ID, Key::StealthAccountV1)?.into_mut();


    // allow the update authority to reset
    if stealth.wallet_pk != *owner_info.key {
        let elgamal_pubkey_info = next_account_info(account_info_iter)?;

        let owner_elgamal_pk = validate_elgamal_pk(
            &elgamal_pubkey_info, &owner_info, &mint_info)?;

        // TODO: does this need to be an equality proof or do we trust the update authority?
        if data.elgamal_pk != owner_elgamal_pk {
            msg!("Not the current owners elgamal pk");
            return Err(StealthError::InvalidElgamalPubkeyPDA.into());
        }

        stealth.wallet_pk = *owner_info.key;
        stealth.elgamal_pk = data.elgamal_pk;
    }


    stealth.encrypted_cipher_key = data.encrypted_cipher_key;
    stealth.uri = data.uri;

    Ok(())
}

// since filling the transfer buffer (even just sending the instruction and if they fail somehow or
// are snooped by someone along the way) fully allows the dest keypair to decrypt, we need to have
// some kind of timeout based transaction fulfillment
//
// some external marketplace program creates an escrow environment (bids only, no instant buy,
// though there certainly can be some kind of timeout crank available)
//
// - buyer places bid, locking funds into some escrow account
// - until bid is accepted, buyer can cancel and reclaim funds
// - bid is marked accepted by the seller
//     - seller commits some (configurable) collateral to escrow
//     - bid funds are locked for X slots
// - before X elapses, the seller does the full transfer and the program releases all funds to the
//   seller once fini is accepted + nft has been transferred. fini should be called through a CPI
//   that also distributes royalties from the escrowed funds
// - otherwise after X elapses, buyer can close the escrow account and claim seller collateral
//
// - potentially marketplace offers centralised handling of the encrypted key so that seller
//   doesn't need to manually handle bids (mimicking a buy-now functionality)
fn process_init_transfer(
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let payer_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let stealth_info = next_account_info(account_info_iter)?;
    let recipient_info = next_account_info(account_info_iter)?;
    let recipient_elgamal_info = next_account_info(account_info_iter)?;
    let transfer_buffer_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let rent_sysvar_info = next_account_info(account_info_iter)?;

    if !payer_info.is_signer {
        return Err(ProgramError::InvalidArgument);
    }
    validate_account_owner(mint_info, &spl_token::ID)?;
    validate_account_owner(token_account_info, &spl_token::ID)?;
    validate_account_owner(stealth_info, &ID)?;

    let token_account = spl_token::state::Account::unpack(
        &token_account_info.data.borrow())?;

    if token_account.mint != *mint_info.key {
        msg!("Mint mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if token_account.owner != *payer_info.key {
        msg!("Owner mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if token_account.amount != 1 {
        msg!("Invalid amount");
        return Err(ProgramError::InvalidArgument);
    }


    // check that stealth matches mint
    let (stealth_key, _stealth_bump_seed) =
        get_stealth_address(mint_info.key);

    if stealth_key != *stealth_info.key {
        return Err(StealthError::InvalidStealthKey.into());
    }

    // deserialize to verify it exists...
    let _stealth = StealthAccount::from_account_info(
        &stealth_info, &ID, Key::StealthAccountV1)?;

    let recipient_elgamal_pk = validate_elgamal_pk(
        &recipient_elgamal_info, &recipient_info, &mint_info)?;

    // check and initialize the cipher key transfer buffer
    let (transfer_buffer_key, transfer_buffer_bump_seed) =
        get_transfer_buffer_address(recipient_info.key, mint_info.key);

    if transfer_buffer_key != *transfer_buffer_info.key {
        msg!("Invalid transfer buffer key");
        return Err(ProgramError::InvalidArgument);
    }

    mpl_token_metadata::utils::create_or_allocate_account_raw(
        ID,
        transfer_buffer_info,
        rent_sysvar_info,
        system_program_info,
        payer_info,
        CipherKeyTransferBuffer::get_packed_len(),
        &[
            TRANSFER.as_bytes(),
            recipient_info.key.as_ref(),
            mint_info.key.as_ref(),
            &[transfer_buffer_bump_seed],
        ],
    )?;

    let mut transfer_buffer = CipherKeyTransferBuffer::from_account_info(
        &transfer_buffer_info, &ID, Key::Uninitialized)?.into_mut();

    // low bits should be clear regardless...
    transfer_buffer.key = Key::CipherKeyTransferBufferV1;
    transfer_buffer.stealth_key = *stealth_info.key;
    transfer_buffer.wallet_pk = *recipient_info.key;
    transfer_buffer.elgamal_pk = recipient_elgamal_pk;

    Ok(())
}

fn process_fini_transfer(
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let authority_info = next_account_info(account_info_iter)?;
    let stealth_info = next_account_info(account_info_iter)?;
    let transfer_buffer_info = next_account_info(account_info_iter)?;
    let _system_program_info = next_account_info(account_info_iter)?;

    if !authority_info.is_signer {
        return Err(ProgramError::InvalidArgument);
    }

    // check that transfer buffer matches passed in arguments and that we have authority to do
    // the transfer
    let transfer_buffer = CipherKeyTransferBuffer::from_account_info(
        &transfer_buffer_info, &ID, Key::CipherKeyTransferBufferV1)?;

    let mut stealth = StealthAccount::from_account_info(
        &stealth_info, &ID, Key::StealthAccountV1)?.into_mut();

    validate_transfer_buffer(
        &transfer_buffer,
        &stealth,
        authority_info.key,
        stealth_info.key,
    )?;

    if !bool::from(&transfer_buffer.updated) {
        msg!("Not all chunks set");
        return Err(ProgramError::InvalidArgument);
    }

    // write the new cipher text over
    stealth.wallet_pk = transfer_buffer.wallet_pk;
    stealth.elgamal_pk = transfer_buffer.elgamal_pk;
    stealth.encrypted_cipher_key = transfer_buffer.encrypted_cipher_key;

    let close_transfer_buffer = || -> ProgramResult {
        let starting_lamports = authority_info.lamports();
        **authority_info.lamports.borrow_mut() = starting_lamports
            .checked_add(transfer_buffer_info.lamports())
            .ok_or::<ProgramError>(StealthError::Overflow.into())?;

        **transfer_buffer_info.lamports.borrow_mut() = 0;
        Ok(())
    };

    close_transfer_buffer()?;

    Ok(())
}

fn process_transfer_chunk<'info>(
    accounts: &[AccountInfo<'info>],
    transfer: &TransferData,
    do_verify: fn (
        &TransferData,
        &Pubkey,
        &mut std::slice::Iter<AccountInfo<'info>>
    ) -> ProgramResult,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let authority_info = next_account_info(account_info_iter)?;
    let stealth_info = next_account_info(account_info_iter)?;
    let transfer_buffer_info = next_account_info(account_info_iter)?;

    if !authority_info.is_signer {
        return Err(ProgramError::InvalidArgument);
    }

    // check that transfer buffer matches passed in arguments and that we have authority to do
    // the transfer
    //
    // TODO: should we have another check for nft ownership here?
    let mut transfer_buffer = CipherKeyTransferBuffer::from_account_info(
        &transfer_buffer_info, &ID, Key::CipherKeyTransferBufferV1)?.into_mut();

    let stealth = StealthAccount::from_account_info(
        &stealth_info, &ID, Key::StealthAccountV1)?;

    validate_transfer_buffer(
        &transfer_buffer,
        &stealth,
        authority_info.key,
        stealth_info.key,
    )?;

    // check that this proof has matching pubkey fields and that we haven't already processed this
    // chunk
    if bool::from(&transfer_buffer.updated) {
        msg!("Chunk already updated");
        return Err(ProgramError::InvalidArgument);
    }

    if transfer.transfer_public_keys.src_pubkey != stealth.elgamal_pk {
        msg!("Source elgamal pubkey mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if transfer.src_cipher_key_chunk_ct != stealth.encrypted_cipher_key {
        msg!("Source cipher text mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if transfer.transfer_public_keys.dst_pubkey != transfer_buffer.elgamal_pk {
        msg!("Destination elgamal pubkey mismatch");
        return Err(ProgramError::InvalidArgument);
    }


    // actually verify the proof...
    // TODO: syscalls when available
    do_verify(&transfer, authority_info.key, account_info_iter)?;

    transfer_buffer.updated = true.into();
    transfer_buffer.encrypted_cipher_key = transfer.dst_cipher_key_chunk_ct;


    Ok(())
}

// syscall not actually available. this will only work in custom / test environments
fn verify_syscall<'info>(
    transfer: &TransferData,
    _authority: &Pubkey,
    _account_info_iter: &mut std::slice::Iter<AccountInfo<'info>>,
) -> ProgramResult {
    transfer.verify().map_err(
        |_| StealthError::ProofVerificationError.into())
}

fn verify_dsl_crank<'info>(
    transfer: &TransferData,
    authority: &Pubkey,
    account_info_iter: &mut std::slice::Iter<AccountInfo<'info>>,
) -> ProgramResult {
    let instruction_buffer_info = next_account_info(account_info_iter)?;
    let input_buffer_info = next_account_info(account_info_iter)?;
    let compute_buffer_info = next_account_info(account_info_iter)?;

    msg!("Verifying compute inputs...");
    use curve25519_dalek_onchain::instruction as dalek;
    use std::borrow::Borrow;
    use borsh::BorshDeserialize;

    validate_account_owner(instruction_buffer_info, &curve25519_dalek_onchain::ID)?;
    validate_account_owner(input_buffer_info, &curve25519_dalek_onchain::ID)?;
    validate_account_owner(compute_buffer_info, &curve25519_dalek_onchain::ID)?;

    let conv_error = || -> ProgramError { StealthError::ProofVerificationError.into() };

    // check that the compute buffer points to the right things
    let compute_buffer_data = compute_buffer_info.try_borrow_data()?;
    let mut compute_buffer_ptr: &[u8] = compute_buffer_data.borrow();
    let compute_buffer_header = dalek::ComputeHeader::deserialize(&mut compute_buffer_ptr)?;
    if dalek::HEADER_SIZE < 128 {
        msg!("Header size seems too small");
        return Err(ProgramError::InvalidArgument);
    }
    if compute_buffer_header.key != dalek::Key::ComputeBufferV1 {
        msg!("Invalid compute buffer type");
        return Err(ProgramError::InvalidArgument);
    }
    if compute_buffer_header.authority != *authority {
        msg!("Invalid compute buffer authority");
        return Err(ProgramError::InvalidArgument);
    }
    if compute_buffer_header.instruction_buffer != *instruction_buffer_info.key {
        msg!("Mismatched instruction buffer");
        return Err(ProgramError::InvalidArgument);
    }
    if compute_buffer_header.input_buffer != *input_buffer_info.key {
        msg!("Mismatched input buffer");
        return Err(ProgramError::InvalidArgument);
    }
    let expected_count: u32 = DSL_INSTRUCTION_COUNT.try_into().map_err(|_| conv_error())?;
    if compute_buffer_header.instruction_num != expected_count {
        msg!("Incomplete compute buffer. {} of {}", compute_buffer_header.instruction_num, expected_count);
        return Err(ProgramError::InvalidArgument);
    }

    // verify that the instruction buffer is correct
    let instruction_buffer_data = instruction_buffer_info.try_borrow_data()?;
    if instruction_buffer_data[dalek::HEADER_SIZE..]
        != DSL_INSTRUCTION_BYTES
    {
        msg!("Invalid instruction buffer");
        return Err(ProgramError::InvalidArgument);
    }

    solana_program::log::sol_log_compute_units();

    /* we expect the input buffer to be laid out as the following:
     *
     * [
     *    // ..input header..
     *
     *    // equality proof statement points
     *    32 bytes:  src elgamal pubkey
     *    32 bytes:  pedersen base H compressed
     *    32 bytes:  Y_0 (b_1 * src elegamal pubkey)
     *
     *    32 bytes:  dst elgamal pubkey
     *    32 bytes:  D2_EG dst cipher text pedersen decrypt handle
     *    32 bytes:  Y_1 (b_2 * dst elegamal pubkey)
     *
     *    32 bytes:  C2_EG dst cipher text pedersen commitment
     *    32 bytes:  C1_EG src cipher text pedersen commitment
     *    32 bytes:  D1_EG src cipher text pedersen decrypt handle
     *    32 bytes:  pedersen base H compressed
     *    32 bytes:  Y_2 (b_1 * src decrypt handle - b_2 * H)
     *
     *
     *    // equality verification scalars
     *    // that s_1 is the secret key for P1_EG
     *    32 bytes:  self.sh_1
     *    32 bytes:  -c
     *    32 bytes:  -Scalar::one()
     *
     *    // that r_2 is the randomness used in D2_EG
     *    32 bytes:  self.rh_2
     *    32 bytes:  -c
     *    32 bytes:  -Scaler::one()
     *
     *    // that the messages in C1_EG and C2_EG are equal under s_1 and r_2
     *    32 bytes:  c
     *    32 bytes:  -c
     *    32 bytes:  self.sh_1
     *    32 bytes:  -self.rh_2
     *    32 bytes:  -Scaler::one()
     *
     *
     */

    let mut buffer_idx = dalek::HEADER_SIZE;
    let input_buffer_data = input_buffer_info.try_borrow_data()?;

    let equality_proof = EqualityProof::from_bytes(&transfer.proof.equality_proof.0)
        .map_err(|_| conv_error())?;

    // verify proof values are as expected
    let expected_pubkeys = [
        // statement inputs
        &transfer.transfer_public_keys.src_pubkey.0,
        &COMPRESSED_H,
        &equality_proof.Y_0.0,

        &transfer.transfer_public_keys.dst_pubkey.0,
        &transfer.dst_cipher_key_chunk_ct.0[32..],
        &equality_proof.Y_1.0,

        &transfer.dst_cipher_key_chunk_ct.0[..32],
        &transfer.src_cipher_key_chunk_ct.0[..32],
        &transfer.src_cipher_key_chunk_ct.0[32..],
        &COMPRESSED_H,
        &equality_proof.Y_2.0,
    ];
    msg!("Verifying input points");
    for i in 0..expected_pubkeys.len() {
        let found_pubkey = &input_buffer_data[buffer_idx..buffer_idx+32];
        if *found_pubkey != *expected_pubkeys[i] {
            msg!("Mismatched proof statement keys");
            return Err(StealthError::ProofVerificationError.into());
        }
        buffer_idx += 32;
    }

    solana_program::log::sol_log_compute_units();

    // same as in TransferProof::verify and EqualityProof::verify but with DSL outputs
    let mut transcript = TransferProof::transcript_new();

    TransferProof::build_transcript(
        &transfer.src_cipher_key_chunk_ct,
        &transfer.dst_cipher_key_chunk_ct,
        &transfer.transfer_public_keys,
        &mut transcript,
    ).map_err(|_| conv_error())?;

    EqualityProof::build_transcript(
        &equality_proof,
        &mut transcript,
    ).map_err(|_| conv_error())?;

    solana_program::log::sol_log_compute_units();

    msg!("Getting challenge scalars");
    let challenge_c = transcript.challenge_scalar(b"c");
    let challenge_w = transcript.challenge_scalar(b"w");
    let challenge_ww = challenge_w  * challenge_w;

    solana_program::log::sol_log_compute_units();

    // verify scalars are as expected
    use curve25519_dalek::scalar::Scalar;
    let neg_challenge_c = -challenge_c;
    let neg_rh_2 = -equality_proof.rh_2;
    let neg_one = Scalar{ bytes: [
        0xEC, 0xD3, 0xF5, 0x5C, 0x1A, 0x63, 0x12, 0x58,
        0xD6, 0x9C, 0xF7, 0xA2, 0xDE, 0xF9, 0xDE, 0x14,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
    ] };
    let expected_scalars = [
         &equality_proof.sh_1,
         &neg_challenge_c,
         &neg_one,

         &(&challenge_w * &equality_proof.rh_2),
         &(&challenge_w * &neg_challenge_c),
         &(&challenge_w * &neg_one),

         &(&challenge_ww * challenge_c),
         &(&challenge_ww * neg_challenge_c),
         &(&challenge_ww * equality_proof.sh_1),
         &(&challenge_ww * neg_rh_2),
         &(&challenge_ww * neg_one),
    ];

    solana_program::log::sol_log_compute_units();

    msg!("Verifying input scalars");
    for i in 0..expected_scalars.len() {
        let mut scalar_buffer = [0; 32];
        scalar_buffer.copy_from_slice(&input_buffer_data[buffer_idx..buffer_idx+32]);
        let packed = curve25519_dalek_onchain::scalar::Scalar{
            bytes: expected_scalars[i].bytes
        }.to_packed_radix_16();
        if scalar_buffer != packed {
            msg!("Mismatched proof statement scalars");
            return Err(StealthError::ProofVerificationError.into());
        }
        buffer_idx += 32;
    }

    solana_program::log::sol_log_compute_units();

    // check that multiplication results are correct
    use curve25519_dalek::traits::IsIdentity;
    let mut buffer_idx = dalek::HEADER_SIZE;
    msg!("Verifying multiscalar mul results");
    for _i in 0..1 {
        let mul_result = curve25519_dalek::edwards::EdwardsPoint::from_bytes(
            &compute_buffer_data[buffer_idx..buffer_idx+128]
        );

        if ! curve25519_dalek::ristretto::RistrettoPoint(mul_result).is_identity() {
            msg!("Proof statement did not verify");
            return Err(StealthError::ProofVerificationError.into());
        }
        buffer_idx += 128;
    }

    Ok(())
}

fn process_publish_elgamal_pubkey(
    accounts: &[AccountInfo],
    elgamal_pk: &zk_token_elgamal::pod::ElGamalPubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let wallet_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let elgamal_pubkey_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let rent_sysvar_info = next_account_info(account_info_iter)?;

    if !wallet_info.is_signer {
        return Err(ProgramError::InvalidArgument);
    }
    validate_account_owner(mint_info, &spl_token::ID)?;

    // check that PDA matches
    let (elgamal_pubkey_key, elgamal_pubkey_bump_seed) =
        get_elgamal_pubkey_address(wallet_info.key, mint_info.key);

    if elgamal_pubkey_key != *elgamal_pubkey_info.key {
        msg!("Invalid wallet elgamal PDA");
        return Err(StealthError::InvalidElgamalPubkeyPDA.into());
    }

    // create and initialize PDA
    mpl_token_metadata::utils::create_or_allocate_account_raw(
        ID,
        elgamal_pubkey_info,
        rent_sysvar_info,
        system_program_info,
        wallet_info,
        EncryptionKeyBuffer::get_packed_len(),
        &[
            PREFIX.as_bytes(),
            wallet_info.key.as_ref(),
            mint_info.key.as_ref(),
            &[elgamal_pubkey_bump_seed],
        ],
    )?;

    let mut encryption_buffer = EncryptionKeyBuffer::from_account_info(
        &elgamal_pubkey_info, &ID, Key::Uninitialized)?.into_mut();

    encryption_buffer.key = Key::EncryptionKeyBufferV1;
    encryption_buffer.owner = *wallet_info.key;
    encryption_buffer.mint = *mint_info.key;
    encryption_buffer.elgamal_pk = *elgamal_pk;

    Ok(())
}

fn process_close_elgamal_pubkey(
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let wallet_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let elgamal_pubkey_info = next_account_info(account_info_iter)?;
    let _system_program_info = next_account_info(account_info_iter)?;

    if !wallet_info.is_signer {
        return Err(ProgramError::InvalidArgument);
    }
    validate_account_owner(mint_info, &spl_token::ID)?;
    validate_account_owner(elgamal_pubkey_info, &ID)?;

    // check that PDA matches
    let (elgamal_pubkey_key, _elgamal_pubkey_bump_seed) =
        get_elgamal_pubkey_address(wallet_info.key, mint_info.key);

    if elgamal_pubkey_key != *elgamal_pubkey_info.key {
        msg!("Invalid wallet elgamal PDA");
        return Err(StealthError::InvalidElgamalPubkeyPDA.into());
    }

    // close the elgamal pubkey buffer
    let starting_lamports = wallet_info.lamports();
    **wallet_info.lamports.borrow_mut() = starting_lamports
        .checked_add(elgamal_pubkey_info.lamports())
        .ok_or::<ProgramError>(StealthError::Overflow.into())?;

    **elgamal_pubkey_info.lamports.borrow_mut() = 0;

    Ok(())
}

fn validate_account_owner(account_info: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    if account_info.owner == owner {
        Ok(())
    } else {
        msg!("Mismatched account owner");
        Err(ProgramError::InvalidArgument)
    }
}

fn validate_transfer_buffer(
    transfer_buffer: &CipherKeyTransferBuffer,
    stealth: &StealthAccount,
    authority: &Pubkey,
    stealth_key: &Pubkey,
) -> ProgramResult {
    if stealth.wallet_pk != *authority {
        msg!("Owner mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if transfer_buffer.stealth_key != *stealth_key {
        msg!("Stealth mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    Ok(())
}

fn validate_metadata(
    metadata_info: &AccountInfo,
    mint_info: &AccountInfo,
) -> Result<mpl_token_metadata::state::Metadata, ProgramError> {
    // check metadata matches mint
    let metadata_seeds = &[
        mpl_token_metadata::state::PREFIX.as_bytes(),
        mpl_token_metadata::ID.as_ref(),
        mint_info.key.as_ref(),
    ];
    let (metadata_key, _metadata_bump_seed) =
        Pubkey::find_program_address(metadata_seeds, &mpl_token_metadata::ID);

    if metadata_key != *metadata_info.key {
        msg!("Invalid metadata key");
        return Err(StealthError::InvalidMetadataKey.into());
    }

    let metadata = mpl_token_metadata::state::Metadata::from_account_info(metadata_info)?;

    if !metadata.is_mutable {
        msg!("Metadata is immutable");
        return Err(StealthError::MetadataIsImmutable.into());
    }

    Ok(metadata)
}

// check that elgamal PDAs match
fn validate_elgamal_pk(
    elgamal_info: &AccountInfo,
    wallet_info: &AccountInfo,
    mint_info: &AccountInfo,
) -> Result<zk_token_elgamal::pod::ElGamalPubkey, ProgramError> {
    let (elgamal_pubkey_key, _elgamal_pubkey_bump_seed) =
        get_elgamal_pubkey_address(wallet_info.key, mint_info.key);

    if elgamal_pubkey_key != *elgamal_info.key {
        msg!("Invalid elgamal PDA");
        return Err(StealthError::InvalidElgamalPubkeyPDA.into());
    }

    let encryption_buffer = EncryptionKeyBuffer::from_account_info(
        &elgamal_info, &ID, Key::EncryptionKeyBufferV1)?;

    Ok(encryption_buffer.elgamal_pk)
}
