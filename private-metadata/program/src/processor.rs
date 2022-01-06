
use crate::{
    instruction::*,
    state::*,
    error::*,
    pod::*,
    transfer_proof::{Verifiable, TransferProof},
    equality_proof::*,
    transcript::TranscriptProtocol,
    ID,
};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_pack::Pack,
    program::{invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::{Sysvar},
};

use crate::{
    zk_token_elgamal,
};

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    match decode_instruction_type(input)? {
        PrivateMetadataInstruction::ConfigureMetadata => {
            msg!("ConfigureMetadata!");
            process_configure_metadata(
                accounts,
                decode_instruction_data::<ConfigureMetadataData>(input)?
            )
        }
        PrivateMetadataInstruction::InitTransfer => {
            msg!("InitTransfer!");
            process_init_transfer(
                accounts,
            )
        }
        PrivateMetadataInstruction::FiniTransfer => {
            msg!("FiniTransfer!");
            process_fini_transfer(
                accounts,
            )
        }
        PrivateMetadataInstruction::TransferChunk => {
            msg!("TransferChunk!");
            process_transfer_chunk(
                accounts,
                decode_instruction_data::<TransferChunkData>(input)?
            )
        }
        PrivateMetadataInstruction::TransferChunkSlow => {
            msg!("TransferChunkSlow!");
            process_transfer_chunk_slow(
                accounts,
                decode_instruction_data::<TransferChunkSlowData>(input)?
            )
        }
        PrivateMetadataInstruction::PublishElgamalPubkey => {
            msg!("PublishElgamalPubkey!");
            process_publish_elgamal_pubkey(
                accounts,
                decode_instruction_data::<zk_token_elgamal::pod::ElGamalPubkey>(input)?
            )
        }
        PrivateMetadataInstruction::CloseElgamalPubkey => {
            msg!("CloseElgamalPubkey!");
            process_close_elgamal_pubkey(
                accounts,
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
    let private_metadata_info = next_account_info(account_info_iter)?;
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
    validate_account_owner(metadata_info, &spl_token_metadata::ID)?;


    // check metadata matches mint
    let metadata_seeds = &[
        spl_token_metadata::state::PREFIX.as_bytes(),
        spl_token_metadata::ID.as_ref(),
        mint_info.key.as_ref(),
    ];
    let (metadata_key, _metadata_bump_seed) =
        Pubkey::find_program_address(metadata_seeds, &spl_token_metadata::ID);

    if metadata_key != *metadata_info.key {
        msg!("Invalid metadata key");
        return Err(PrivateMetadataError::InvalidMetadataKey.into());
    }


    // check that metadata authority matches and that metadata is mutable (adding private metadata
    // and not acting on a limited edition). TODO?
    let metadata = spl_token_metadata::state::Metadata::from_account_info(metadata_info)?;

    let authority_pubkey = metadata.update_authority;

    if authority_pubkey != *metadata_update_authority_info.key {
        msg!("Invalid metadata update authority");
        return Err(PrivateMetadataError::InvalidUpdateAuthority.into());
    }

    if !metadata.is_mutable {
        msg!("Metadata is immutable");
        return Err(PrivateMetadataError::MetadataIsImmutable.into());
    }


    // check that private metadata matches mint
    let private_metadata_seeds = &[
        PREFIX.as_bytes(),
        mint_info.key.as_ref(),
    ];
    let (private_metadata_key, private_metadata_bump_seed) =
        Pubkey::find_program_address(private_metadata_seeds, &ID);

    if private_metadata_key != *private_metadata_info.key {
        msg!("Invalid private metadata key");
        return Err(PrivateMetadataError::InvalidPrivateMetadataKey.into());
    }


    // create and initialize PDA
    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    invoke_signed(
        &system_instruction::create_account(
            payer_info.key,
            private_metadata_info.key,
            rent.minimum_balance(PrivateMetadataAccount::get_packed_len()).max(1),
            PrivateMetadataAccount::get_packed_len() as u64,
            &ID,
        ),
        &[
            payer_info.clone(),
            private_metadata_info.clone(),
            system_program_info.clone(),
        ],
        &[
            &[
                PREFIX.as_bytes(),
                mint_info.key.as_ref(),
                &[private_metadata_bump_seed],
            ],
        ],
    )?;

    let mut private_metadata = PrivateMetadataAccount::from_account_info(
        &private_metadata_info, &ID)?.into_mut();

    private_metadata.key = Key::PrivateMetadataAccountV1;
    private_metadata.mint = *mint_info.key;
    private_metadata.elgamal_pk = data.elgamal_pk;
    private_metadata.encrypted_cipher_key = data.encrypted_cipher_key;
    private_metadata.uri = data.uri;

    Ok(())
}

// TODO: since creating filling the transfer buffer (even just sending the instruction and if they
// fail somehow or are snooped by someone along the way) fully allows the dest keypair to decrypt
// so it needs to be some handshake process i think...
//
// can this be a separate program?
//
// - bid is marked accepted by the seller
//     - seller commits some portion to escrow (10%?)
//     - bid funds are locked for period X
// - before X elapses, the seller does the full transfer and the program releases all funds to the
//   seller once fini is accepted + nft has been transferred
// - after X, buyer can show key has not yet been transfered and claim their funds back along with
//   the seller escrow
//
// i think this means that only 1 sale can happen at a time? which does seem correct since their is
// only 1 and this 'atomic' operation is kind of split

fn process_init_transfer(
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let authority_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let private_metadata_info = next_account_info(account_info_iter)?;
    let recipient_info = next_account_info(account_info_iter)?;
    let recipient_elgamal_info = next_account_info(account_info_iter)?;
    let transfer_buffer_info = next_account_info(account_info_iter)?;
    let _system_program_info = next_account_info(account_info_iter)?;
    let rent_sysvar_info = next_account_info(account_info_iter)?;

    if !authority_info.is_signer {
        return Err(ProgramError::InvalidArgument);
    }
    validate_account_owner(mint_info, &spl_token::ID)?;
    validate_account_owner(token_account_info, &spl_token::ID)?;
    validate_account_owner(private_metadata_info, &ID)?;
    validate_account_owner(transfer_buffer_info, &ID)?;

    let token_account = spl_token::state::Account::unpack(
        &token_account_info.data.borrow())?;

    if token_account.mint != *mint_info.key {
        msg!("Mint mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if token_account.owner != *authority_info.key {
        msg!("Owner mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    // TODO: this is a bit fucky since the nft token transfer should really happen at the same time
    // as the private metadata transfer...
    if token_account.amount != 1 {
        msg!("Invalid amount");
        return Err(ProgramError::InvalidArgument);
    }


    // check that private metadata matches mint
    let private_metadata_seeds = &[
        PREFIX.as_bytes(),
        mint_info.key.as_ref(),
    ];
    let (private_metadata_key, _private_metadata_bump_seed) =
        Pubkey::find_program_address(private_metadata_seeds, &ID);

    if private_metadata_key != *private_metadata_info.key {
        return Err(PrivateMetadataError::InvalidPrivateMetadataKey.into());
    }

    // check that elgamal PDA matches
    let elgamal_seeds = &[
        PREFIX.as_bytes(),
        recipient_info.key.as_ref(),
        mint_info.key.as_ref(),
    ];
    let (elgamal_pubkey_key, _elgamal_pubkey_bump_seed) =
        Pubkey::find_program_address(elgamal_seeds, &ID);

    if elgamal_pubkey_key != *recipient_elgamal_info.key {
        msg!("Invalid recipient elgamal PDA");
        return Err(PrivateMetadataError::InvalidElgamalPubkeyPDA.into());
    }

    use std::convert::TryInto;
    let elgamal_pk = zk_token_elgamal::pod::ElGamalPubkey(
        (*recipient_elgamal_info.try_borrow_data()?)
            .as_ref()
            .try_into()
            .map_err(|_| -> ProgramError {
                msg!("Invalid recipient elgamal PDA data");
                PrivateMetadataError::InvalidElgamalPubkeyPDA.into()
            })?
    );


    // check and initialize the cipher key transfer buffer
    if transfer_buffer_info.data_len()
            != CipherKeyTransferBuffer::get_packed_len() {
        return Err(ProgramError::AccountDataTooSmall);
    }

    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    if transfer_buffer_info.lamports()
            < rent.minimum_balance(CipherKeyTransferBuffer::get_packed_len()).max(1) {
        return Err(ProgramError::InsufficientFunds);
    }

    let mut transfer_buffer = CipherKeyTransferBuffer::from_account_info(
        &transfer_buffer_info, &ID)?.into_mut();

    if transfer_buffer.key != Key::Uninitialized {
        msg!("Transfer buffer already initialized");
        return Err(PrivateMetadataError::BufferAlreadyInitialized.into());
    }

    // low bits should be clear regardless...
    transfer_buffer.key = Key::CipherKeyTransferBufferV1;
    transfer_buffer.authority = *authority_info.key;
    transfer_buffer.private_metadata_key = *private_metadata_info.key;
    transfer_buffer.elgamal_pk = elgamal_pk;

    Ok(())
}

// TODO: this should be cheap and should be bundled with the actual NFT transfer
fn process_fini_transfer(
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let authority_info = next_account_info(account_info_iter)?;
    let private_metadata_info = next_account_info(account_info_iter)?;
    let transfer_buffer_info = next_account_info(account_info_iter)?;
    let _system_program_info = next_account_info(account_info_iter)?;

    if !authority_info.is_signer {
        return Err(ProgramError::InvalidArgument);
    }
    validate_account_owner(private_metadata_info, &ID)?;
    validate_account_owner(transfer_buffer_info, &ID)?;

    // check that transfer buffer matches passed in arguments and that we have authority to do
    // the transfer
    //
    // TODO: should we have a nother check for nft ownership here?
    let transfer_buffer = CipherKeyTransferBuffer::from_account_info(
        &transfer_buffer_info, &ID)?;

    if transfer_buffer.key != Key::CipherKeyTransferBufferV1 { // redundant?
        msg!("Mismatched transfer buffer key");
        return Err(ProgramError::InvalidArgument);
    }

    if transfer_buffer.authority != *authority_info.key {
        msg!("Owner mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if transfer_buffer.private_metadata_key!= *private_metadata_info.key {
        msg!("Private metadata mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if !bool::from(&transfer_buffer.updated) {
        msg!("Not all chunks set");
        return Err(ProgramError::InvalidArgument);
    }


    // write the new cipher text over
    let mut private_metadata = PrivateMetadataAccount::from_account_info(
        &private_metadata_info, &ID)?.into_mut();

    if private_metadata.key != Key::PrivateMetadataAccountV1 { // redundant?
        msg!("Mismatched private metadata key");
        return Err(ProgramError::InvalidArgument);
    }

    private_metadata.elgamal_pk = transfer_buffer.elgamal_pk;
    private_metadata.encrypted_cipher_key = transfer_buffer.encrypted_cipher_key;


    // close the transfer buffer
    let starting_lamports = authority_info.lamports();
    **authority_info.lamports.borrow_mut() = starting_lamports
        .checked_add(transfer_buffer_info.lamports())
        .ok_or::<ProgramError>(PrivateMetadataError::Overflow.into())?;

    **transfer_buffer_info.lamports.borrow_mut() = 0;

    Ok(())
}

fn process_transfer_chunk(
    accounts: &[AccountInfo],
    data: &TransferChunkData,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let authority_info = next_account_info(account_info_iter)?;
    let private_metadata_info = next_account_info(account_info_iter)?;
    let transfer_buffer_info = next_account_info(account_info_iter)?;
    let _system_program_info = next_account_info(account_info_iter)?;

    if !authority_info.is_signer {
        return Err(ProgramError::InvalidArgument);
    }
    validate_account_owner(private_metadata_info, &ID)?;
    validate_account_owner(transfer_buffer_info, &ID)?;

    // check that transfer buffer matches passed in arguments and that we have authority to do
    // the transfer
    //
    // TODO: should we have a nother check for nft ownership here?
    // TODO: consolidate with fini
    let mut transfer_buffer = CipherKeyTransferBuffer::from_account_info(
        &transfer_buffer_info, &ID)?.into_mut();

    if transfer_buffer.key != Key::CipherKeyTransferBufferV1 {
        msg!("Transfer buffer not initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    if transfer_buffer.authority != *authority_info.key {
        msg!("Owner mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if transfer_buffer.private_metadata_key != *private_metadata_info.key {
        msg!("Private metadata mismatch");
        return Err(ProgramError::InvalidArgument);
    }


    // check that this proof has matching pubkey fields and that we haven't already processed this
    // chunk
    if bool::from(&transfer_buffer.updated) {
        msg!("Chunk already updated");
        return Err(ProgramError::InvalidArgument);
    }

    // TODO: don't deserialize the whole thing
    let private_metadata = PrivateMetadataAccount::from_account_info(
        &private_metadata_info, &ID)?;

    if private_metadata.key != Key::PrivateMetadataAccountV1 { // redundant?
        msg!("Mismatched private metadata key");
        return Err(ProgramError::InvalidArgument);
    }

    let transfer = &data.transfer;
    if transfer.transfer_public_keys.src_pubkey != private_metadata.elgamal_pk {
        msg!("Source elgamal pubkey mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if transfer.src_cipher_key_chunk_ct != private_metadata.encrypted_cipher_key {
        msg!("Source cipher text mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if transfer.transfer_public_keys.dst_pubkey != transfer_buffer.elgamal_pk {
        msg!("Destination elgamal pubkey mismatch");
        return Err(ProgramError::InvalidArgument);
    }


    // actually verify the proof...
    // TODO: make this work with bpf
    if transfer.verify().is_err() {
        return Err(PrivateMetadataError::ProofVerificationError.into());
    }

    transfer_buffer.updated = true.into();
    transfer_buffer.encrypted_cipher_key = transfer.dst_cipher_key_chunk_ct;


    Ok(())
}

fn process_transfer_chunk_slow(
    accounts: &[AccountInfo],
    data: &TransferChunkSlowData,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let authority_info = next_account_info(account_info_iter)?;
    let private_metadata_info = next_account_info(account_info_iter)?;
    let transfer_buffer_info = next_account_info(account_info_iter)?;
    let instruction_buffer_info = next_account_info(account_info_iter)?;
    let input_buffer_info = next_account_info(account_info_iter)?;
    let compute_buffer_info = next_account_info(account_info_iter)?;
    let _system_program_info = next_account_info(account_info_iter)?;

    msg!("Verifying transfer inputs...");

    if !authority_info.is_signer {
        return Err(ProgramError::InvalidArgument);
    }
    validate_account_owner(private_metadata_info, &ID)?;
    validate_account_owner(transfer_buffer_info, &ID)?;

    // check that transfer buffer matches passed in arguments and that we have authority to do
    // the transfer
    //
    // TODO: should we have a nother check for nft ownership here?
    // TODO: consolidate with fini
    let mut transfer_buffer = CipherKeyTransferBuffer::from_account_info(
        &transfer_buffer_info, &ID)?.into_mut();

    if transfer_buffer.key != Key::CipherKeyTransferBufferV1 {
        msg!("Transfer buffer not initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    if transfer_buffer.authority != *authority_info.key {
        msg!("Owner mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if transfer_buffer.private_metadata_key != *private_metadata_info.key {
        msg!("Private metadata mismatch");
        return Err(ProgramError::InvalidArgument);
    }


    // check that this proof has matching pubkey fields and that we haven't already processed this
    // chunk
    if bool::from(&transfer_buffer.updated) {
        msg!("Chunk already updated");
        return Err(ProgramError::InvalidArgument);
    }

    // TODO: don't deserialize the whole thing
    let private_metadata = PrivateMetadataAccount::from_account_info(
        &private_metadata_info, &ID)?;

    if private_metadata.key != Key::PrivateMetadataAccountV1 { // redundant?
        msg!("Mismatched private metadata key");
        return Err(ProgramError::InvalidArgument);
    }

    let transfer = &data.transfer;
    if transfer.transfer_public_keys.src_pubkey != private_metadata.elgamal_pk {
        msg!("Source elgamal pubkey mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if transfer.src_cipher_key_chunk_ct != private_metadata.encrypted_cipher_key {
        msg!("Source cipher text mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if transfer.transfer_public_keys.dst_pubkey != transfer_buffer.elgamal_pk {
        msg!("Destination elgamal pubkey mismatch");
        return Err(ProgramError::InvalidArgument);
    }




    msg!("Verifying comopute inputs...");
    use curve25519_dalek_onchain::instruction as dalek;
    use std::borrow::Borrow;
    use std::convert::TryInto;
    use borsh::BorshDeserialize;

    validate_account_owner(instruction_buffer_info, &curve25519_dalek_onchain::ID)?;
    validate_account_owner(input_buffer_info, &curve25519_dalek_onchain::ID)?;
    validate_account_owner(compute_buffer_info, &curve25519_dalek_onchain::ID)?;

    let conv_error = || -> ProgramError { PrivateMetadataError::ProofVerificationError.into() };

    // check that the compute buffer points to the right things
    let compute_buffer_data = compute_buffer_info.try_borrow_data()?;
    let mut compute_buffer_ptr: &[u8] = compute_buffer_data.borrow();
    let compute_buffer_header = dalek::ComputeHeader::deserialize(&mut compute_buffer_ptr)?;
    if dalek::HEADER_SIZE < 128 {
        msg!("Header size seems too small");
        return Err(ProgramError::InvalidArgument);
    }
    if compute_buffer_header.authority != *authority_info.key {
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
            return Err(PrivateMetadataError::ProofVerificationError.into());
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
    // TODO: do we need to fetch 'w'? should be deterministically after...

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

         &equality_proof.rh_2,
         &neg_challenge_c,
         &neg_one,

         &challenge_c,
         &neg_challenge_c,
         &equality_proof.sh_1,
         &neg_rh_2,
         &neg_one,
    ];

    solana_program::log::sol_log_compute_units();

    msg!("Verifying input scalars");
    for i in 0..expected_scalars.len() {
        let mut scalar_buffer = [0; 32];
        scalar_buffer.copy_from_slice(&input_buffer_data[buffer_idx..buffer_idx+32]);
        // TODO: seems a bit weird that this doesn't go through from_canonical_bytes but we're just
        // comparing exact equality...
        let found_scalar = Scalar{ bytes: scalar_buffer };
        if found_scalar.bytes != expected_scalars[i].bytes {
            msg!("Mismatched proof statement scalars");
            return Err(PrivateMetadataError::ProofVerificationError.into());
        }
        buffer_idx += 32;
    }

    solana_program::log::sol_log_compute_units();

    // check that multiplication results are correct
    use curve25519_dalek::traits::IsIdentity;
    let mut buffer_idx = dalek::HEADER_SIZE;
    msg!("Verifying multiscalar mul results");
    for _i in 0..3 {
        let mul_result = curve25519_dalek::edwards::EdwardsPoint::from_bytes(
            &compute_buffer_data[buffer_idx..buffer_idx+128]
        );

        if ! curve25519_dalek::ristretto::RistrettoPoint(mul_result).is_identity() {
            msg!("Proof statement did not verify");
            return Err(PrivateMetadataError::ProofVerificationError.into());
        }
        buffer_idx += 128;
    }

    transfer_buffer.updated = true.into();
    transfer_buffer.encrypted_cipher_key = transfer.dst_cipher_key_chunk_ct;


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
    let seeds = &[
        PREFIX.as_bytes(),
        wallet_info.key.as_ref(),
        mint_info.key.as_ref(),
    ];
    let (elgamal_pubkey_key, elgamal_pubkey_bump_seed) =
        Pubkey::find_program_address(seeds, &ID);

    if elgamal_pubkey_key != *elgamal_pubkey_info.key {
        msg!("Invalid wallet elgamal PDA");
        return Err(PrivateMetadataError::InvalidElgamalPubkeyPDA.into());
    }

    // create and initialize PDA
    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    let space = std::mem::size_of::<zk_token_elgamal::pod::ElGamalPubkey>();
    invoke_signed(
        &system_instruction::create_account(
            wallet_info.key,
            elgamal_pubkey_info.key,
            rent.minimum_balance(space).max(1),
            space as u64,
            &ID,
        ),
        &[
            wallet_info.clone(),
            elgamal_pubkey_info.clone(),
            system_program_info.clone(),
        ],
        &[
            &[
                PREFIX.as_bytes(),
                wallet_info.key.as_ref(),
                mint_info.key.as_ref(),
                &[elgamal_pubkey_bump_seed],
            ],
        ],
    )?;

    let mut elgamal_pubkey_data = elgamal_pubkey_info.try_borrow_mut_data()?;
    elgamal_pubkey_data.copy_from_slice(
        bytemuck::cast_slice::<zk_token_elgamal::pod::ElGamalPubkey, u8>(
            std::slice::from_ref(elgamal_pk)
        )
    );

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
    let seeds = &[
        PREFIX.as_bytes(),
        wallet_info.key.as_ref(),
        mint_info.key.as_ref(),
    ];
    let (elgamal_pubkey_key, _elgamal_pubkey_bump_seed) =
        Pubkey::find_program_address(seeds, &ID);

    if elgamal_pubkey_key != *elgamal_pubkey_info.key {
        msg!("Invalid wallet elgamal PDA");
        return Err(PrivateMetadataError::InvalidElgamalPubkeyPDA.into());
    }

    // close the elgamal pubkey buffer
    let starting_lamports = wallet_info.lamports();
    **wallet_info.lamports.borrow_mut() = starting_lamports
        .checked_add(elgamal_pubkey_info.lamports())
        .ok_or::<ProgramError>(PrivateMetadataError::Overflow.into())?;

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

