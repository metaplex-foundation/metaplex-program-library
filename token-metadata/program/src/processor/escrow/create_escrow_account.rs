use borsh::BorshSerialize;
use mpl_utils::{assert_signer, create_or_allocate_account_raw};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_memory::sol_memcpy,
    pubkey::Pubkey,
    system_program,
};

use super::find_escrow_seeds;
use crate::{
    assertions::{assert_derivation, assert_initialized, assert_owned_by},
    error::MetadataError,
    pda::{EDITION, PREFIX},
    state::{
        EscrowAuthority, Key, Metadata, TokenMetadataAccount, TokenOwnedEscrow, TokenStandard,
    },
    utils::check_token_standard,
};

pub fn process_create_escrow_account(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let escrow_account_info = next_account_info(account_info_iter)?;
    if escrow_account_info.owner != &system_program::ID || !escrow_account_info.data_is_empty() {
        return Err(MetadataError::AlreadyInitialized.into());
    }

    let metadata_account_info = next_account_info(account_info_iter)?;
    assert_owned_by(metadata_account_info, &crate::ID)?;

    let mint_account_info = next_account_info(account_info_iter)?;
    assert_owned_by(mint_account_info, &spl_token::ID)?;

    let token_account_info = next_account_info(account_info_iter)?;
    assert_owned_by(token_account_info, &spl_token::ID)?;

    let edition_account_info = next_account_info(account_info_iter)?;
    assert_owned_by(edition_account_info, &crate::ID)?;

    let payer_account_info = next_account_info(account_info_iter)?;
    assert_signer(payer_account_info)?;

    let system_account_info = next_account_info(account_info_iter)?;
    if *system_account_info.key != system_program::ID {
        return Err(MetadataError::InvalidSystemProgram.into());
    }

    let sysvar_ix_account_info = next_account_info(account_info_iter)?;
    if sysvar_ix_account_info.key != &solana_program::sysvar::instructions::ID {
        return Err(MetadataError::InvalidInstructionsSysvar.into());
    }

    let is_using_authority = account_info_iter.len() == 1;

    let maybe_authority_info: Option<&AccountInfo> = if is_using_authority {
        Some(next_account_info(account_info_iter)?)
    } else {
        None
    };

    let metadata: Metadata = Metadata::from_account_info(metadata_account_info)?;

    // Mint account passed in must be the mint of the metadata account passed in.
    if &metadata.mint != mint_account_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    // Only standard or programmable non-fungible tokens (i.e. unique) can have escrow accounts.
    let token_standard = check_token_standard(mint_account_info, Some(edition_account_info))?;
    if !matches!(
        token_standard,
        TokenStandard::NonFungible | TokenStandard::ProgrammableNonFungible
    ) {
        return Err(MetadataError::MustBeNonFungible.into());
    }

    // Check that the edition account is for this mint.
    let _edition_bump = assert_derivation(
        &crate::ID,
        edition_account_info,
        &[
            PREFIX.as_bytes(),
            crate::ID.as_ref(),
            mint_account_info.key.as_ref(),
            EDITION.as_bytes(),
        ],
    )?;

    let creator = maybe_authority_info.unwrap_or(payer_account_info);
    assert_signer(creator)?;

    let token_account: spl_token::state::Account = assert_initialized(token_account_info)?;

    if token_account.mint != *mint_account_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    if token_account.amount < 1 {
        return Err(MetadataError::NotEnoughTokens.into());
    }

    if token_account.mint != metadata.mint {
        return Err(MetadataError::MintMismatch.into());
    }

    let creator_type = if token_account.owner == *creator.key {
        EscrowAuthority::TokenOwner
    } else {
        EscrowAuthority::Creator(*creator.key)
    };

    // Derive the seeds for PDA signing.
    let escrow_seeds = find_escrow_seeds(mint_account_info.key, &creator_type);

    let bump_seed = &[assert_derivation(
        &crate::ID,
        escrow_account_info,
        &escrow_seeds,
    )?];

    let escrow_authority_seeds = [escrow_seeds, vec![bump_seed]].concat();

    // Initialize a default (empty) escrow structure.
    let toe = TokenOwnedEscrow {
        key: Key::TokenOwnedEscrow,
        base_token: *mint_account_info.key,
        authority: creator_type,
        bump: bump_seed[0],
    };

    let serialized_data = toe
        .try_to_vec()
        .map_err(|_| MetadataError::BorshSerializationError)?;

    // Create the account.
    create_or_allocate_account_raw(
        crate::ID,
        escrow_account_info,
        system_account_info,
        payer_account_info,
        serialized_data.len(),
        &escrow_authority_seeds,
    )?;

    sol_memcpy(
        &mut escrow_account_info.try_borrow_mut_data()?,
        &serialized_data,
        serialized_data.len(),
    );

    Ok(())
}
