use borsh::BorshSerialize;
use mpl_utils::create_or_allocate_account_raw;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};
use spl_token::state::Mint;

use crate::{
    assertions::{
        assert_derivation, assert_initialized, assert_mint_authority_matches_mint, assert_owned_by,
        assert_token_program_matches_package, metadata::assert_update_authority_is_correct,
    },
    error::MetadataError,
    state::{
        Key, MasterEditionV2, Metadata, TokenMetadataAccount, TokenStandard, EDITION,
        MAX_MASTER_EDITION_LEN, PREFIX,
    },
    utils::transfer_mint_authority,
};

/// Create master edition
pub fn process_create_master_edition(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    max_supply: Option<u64>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let edition_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let payer_account_info = next_account_info(account_info_iter)?;
    let metadata_account_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;

    let metadata = Metadata::from_account_info(metadata_account_info)?;
    let mint: Mint = assert_initialized(mint_info)?;

    let bump_seed = assert_derivation(
        program_id,
        edition_account_info,
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            mint_info.key.as_ref(),
            EDITION.as_bytes(),
        ],
    )?;

    assert_token_program_matches_package(token_program_info)?;
    assert_mint_authority_matches_mint(&mint.mint_authority, mint_authority_info)?;
    assert_owned_by(metadata_account_info, program_id)?;
    assert_owned_by(mint_info, &spl_token::id())?;

    if metadata.mint != *mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    if mint.decimals != 0 {
        return Err(MetadataError::EditionMintDecimalsShouldBeZero.into());
    }

    assert_update_authority_is_correct(&metadata, update_authority_info)?;

    if mint.supply != 1 {
        return Err(MetadataError::EditionsMustHaveExactlyOneToken.into());
    }

    let edition_authority_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint_info.key.as_ref(),
        EDITION.as_bytes(),
        &[bump_seed],
    ];

    create_or_allocate_account_raw(
        *program_id,
        edition_account_info,
        system_account_info,
        payer_account_info,
        MAX_MASTER_EDITION_LEN,
        edition_authority_seeds,
    )?;

    let mut edition = MasterEditionV2::from_account_info(edition_account_info)?;

    edition.key = Key::MasterEditionV2;
    edition.supply = 0;
    edition.max_supply = max_supply;
    edition.serialize(&mut *edition_account_info.try_borrow_mut_data()?)?;
    if metadata_account_info.is_writable {
        let mut metadata_mut = Metadata::from_account_info(metadata_account_info)?;
        metadata_mut.token_standard = Some(TokenStandard::NonFungible);
        metadata_mut.serialize(&mut *metadata_account_info.try_borrow_mut_data()?)?;
    }

    // While you can't mint any more of your master record, you can
    // mint as many limited editions as you like within your max supply.
    transfer_mint_authority(
        edition_account_info.key,
        edition_account_info,
        mint_info,
        mint_authority_info,
        token_program_info,
    )?;

    Ok(())
}
