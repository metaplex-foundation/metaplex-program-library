use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_option::COption,
    program_pack::Pack, pubkey::Pubkey,
};
use spl_token::state::Mint;

use crate::{
    error::MetadataError,
    pda::find_master_edition_account,
    state::{MasterEditionV1, EDITION, PREFIX},
};

pub fn assert_edition_is_not_mint_authority(mint_account_info: &AccountInfo) -> ProgramResult {
    let mint = Mint::unpack_from_slice(&mint_account_info.try_borrow_mut_data()?)?;

    let (edition_pda, _) = find_master_edition_account(mint_account_info.key);

    if mint.mint_authority == COption::Some(edition_pda) {
        return Err(MetadataError::MissingEditionAccount.into());
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

pub fn assert_supply_invariance(
    master_edition: &MasterEditionV1,
    printing_mint: &Mint,
    new_supply: u64,
) -> ProgramResult {
    // The supply of printed tokens and the supply of the master edition should, when added, never exceed max supply.
    // Every time a printed token is burned, master edition.supply goes up by 1.
    if let Some(max_supply) = master_edition.max_supply {
        let current_supply = printing_mint
            .supply
            .checked_add(master_edition.supply)
            .ok_or(MetadataError::NumericalOverflowError)?;
        let new_proposed_supply = current_supply
            .checked_add(new_supply)
            .ok_or(MetadataError::NumericalOverflowError)?;
        if new_proposed_supply > max_supply {
            return Err(MetadataError::PrintingWouldBreachMaximumSupply.into());
        }
    }

    Ok(())
}
