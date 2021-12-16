
use crate::{
    instruction::*,
    state::*,
    error::*,
    pod::*,
    transfer_proof::{self, Verifiable},
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

use spl_token_metadata::{
    state::MAX_METADATA_LEN,
};

use spl_zk_token_sdk::{
    zk_token_elgamal,
};

use arrayref::{array_ref};

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
                decode_instruction_data::<zk_token_elgamal::pod::ElGamalPubkey>(input)?
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
        return Err(PrivateMetadataError::InvalidMetadataKey.into());
    }


    // check that metadata authority matches and that metadata is mutable (adding private metadata
    // and not acting on a limited edition). TODO?
    let metadata_data = metadata_info.try_borrow_data()?;

    let authority_data = array_ref![metadata_data, 1, 32];
    let authority_pubkey = Pubkey::new_from_array(*authority_data);

    if authority_pubkey != *metadata_update_authority_info.key {
        return Err(PrivateMetadataError::InvalidUpdateAuthority.into());
    }

    let is_mutable = metadata_data[MAX_METADATA_LEN - 172 - 9 - 1];
    if is_mutable != 0x01 {
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

    private_metadata.mint = *mint_info.key;
    private_metadata.elgamal_pk = data.elgamal_pk;
    private_metadata.encrypted_cipher_key = data.encrypted_cipher_key;
    private_metadata.uri = data.uri;

    Ok(())
}

fn process_init_transfer(
    accounts: &[AccountInfo],
    elgamal_pk: &zk_token_elgamal::pod::ElGamalPubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let authority_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let private_metadata_info = next_account_info(account_info_iter)?;
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

    if (transfer_buffer.updated & BUFFER_INITIALIZED_MASK) != 0 {
        return Err(PrivateMetadataError::BufferAlreadyInitialized.into());
    }
    // low bits should be clear regardless...
    transfer_buffer.updated = BUFFER_INITIALIZED_MASK;
    transfer_buffer.authority = *authority_info.key;
    transfer_buffer.private_metadata_key = *private_metadata_info.key;
    transfer_buffer.elgamal_pk = *elgamal_pk;

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

    if transfer_buffer.authority != *authority_info.key {
        msg!("Owner mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if transfer_buffer.private_metadata_key!= *private_metadata_info.key {
        msg!("Private metadata mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if (transfer_buffer.updated & BUFFER_INITIALIZED_MASK) != BUFFER_INITIALIZED_MASK {
        msg!("Transfer buffer not initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    let all_chunks_set_mask = 1<<CIPHER_KEY_CHUNKS - 1;
    if (transfer_buffer.updated & all_chunks_set_mask) != all_chunks_set_mask {
        msg!("Not all chunks set");
        return Err(ProgramError::InvalidArgument);
    }


    // write the new cipher text over
    let mut private_metadata = PrivateMetadataAccount::from_account_info(
        &private_metadata_info, &ID)?.into_mut();

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

    if transfer_buffer.authority != *authority_info.key {
        msg!("Owner mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if transfer_buffer.private_metadata_key!= *private_metadata_info.key {
        msg!("Private metadata mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if (transfer_buffer.updated & BUFFER_INITIALIZED_MASK) != BUFFER_INITIALIZED_MASK {
        msg!("Transfer buffer not initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    let all_chunks_set_mask = 1<<CIPHER_KEY_CHUNKS - 1;
    if (transfer_buffer.updated & all_chunks_set_mask) != all_chunks_set_mask {
        msg!("Not all chunks set");
        return Err(ProgramError::InvalidArgument);
    }


    // check that this proof has matching pubkey fields and that we haven't already processed this
    // chunk
    let updated_mask = 1<<data.chunk_idx;
    if (transfer_buffer.updated & updated_mask) != 0 {
        msg!("Chunk already updated");
        return Err(ProgramError::InvalidArgument);
    }

    // TODO: don't deserialize the whole thing
    let private_metadata = PrivateMetadataAccount::from_account_info(
        &private_metadata_info, &ID)?;
    let transfer = &data.transfer;
    let chunk_idx: usize = data.chunk_idx.into();

    if transfer.transfer_public_keys.src_pubkey != private_metadata.elgamal_pk {
        msg!("Source elgamal pubkey mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if transfer.src_cipher_key_chunk_ct != private_metadata.encrypted_cipher_key[chunk_idx] {
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

    transfer_buffer.updated |= updated_mask;
    transfer_buffer.encrypted_cipher_key[chunk_idx] = transfer.dst_cipher_key_chunk_ct;


    Ok(())
}

fn validate_account_owner(account_info: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    if account_info.owner == owner {
        Ok(())
    } else {
        Err(ProgramError::InvalidArgument)
    }
}

// TODO: how do we ensure that the encryption key is actually re-encrypted on transfers? can we
// lock some sol and poke the auditor?
//
// so something like the person selling the nft w/ private metadata transfers the token to our
// contract and then we do an auction and then the seller needs to do encrypt the data at that
// point with the pubkey of the winner and then the winner does a transaction where they answer
// the challenge...
