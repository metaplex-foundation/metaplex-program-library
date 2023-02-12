use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_option::COption,
    program_pack::Pack, pubkey::Pubkey,
};
use spl_token::state::Mint;

use crate::{
    error::MetadataError,
    pda::find_master_edition_account,
    state::{TokenStandard, EDITION, PREFIX, TOKEN_STANDARD_INDEX},
};

pub fn assert_edition_is_not_mint_authority(mint_account_info: &AccountInfo) -> ProgramResult {
    let mint = Mint::unpack_from_slice(&mint_account_info.try_borrow_data()?)?;

    let (edition_pda, _) = find_master_edition_account(mint_account_info.key);

    if mint.mint_authority == COption::Some(edition_pda) {
        return Err(MetadataError::MissingEditionAccount.into());
    }

    Ok(())
}

/// Checks that the `master_edition` is not a pNFT master edition.
pub fn assert_edition_is_not_programmable(master_edition_info: &AccountInfo) -> ProgramResult {
    let edition_data = master_edition_info.data.borrow();

    if edition_data.len() > TOKEN_STANDARD_INDEX
        && edition_data[TOKEN_STANDARD_INDEX] == TokenStandard::ProgrammableNonFungible as u8
    {
        return Err(MetadataError::InvalidTokenStandard.into());
    }

    Ok(())
}

// Todo deprecate this for assert derivation
pub fn assert_edition_valid(
    program_id: &Pubkey,
    mint: &Pubkey,
    edition_account_info: &AccountInfo,
) -> ProgramResult {
    let edition_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint.as_ref(),
        EDITION.as_bytes(),
    ];
    let (edition_key, _) = Pubkey::find_program_address(edition_seeds, program_id);
    if edition_key != *edition_account_info.key {
        return Err(MetadataError::InvalidEditionKey.into());
    }

    Ok(())
}
