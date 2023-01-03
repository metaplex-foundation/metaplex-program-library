use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};
use spl_token_2022::state::Mint;

use crate::{
    assertions::{assert_owned_by, assert_owner_in, token_unpack},
    error::MetadataError,
    state::{Key, MasterEditionV1, MasterEditionV2, TokenMetadataAccount},
};

pub fn process_convert_master_edition_v1_to_v2(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let master_edition_info = next_account_info(account_info_iter)?;
    let one_time_printing_auth_mint_info = next_account_info(account_info_iter)?;
    let printing_mint_info = next_account_info(account_info_iter)?;

    assert_owned_by(master_edition_info, program_id)?;
    assert_owner_in(
        one_time_printing_auth_mint_info,
        &mpl_utils::token::TOKEN_PROGRAM_IDS,
    )?;
    assert_owner_in(printing_mint_info, &mpl_utils::token::TOKEN_PROGRAM_IDS)?;
    let master_edition = MasterEditionV1::from_account_info(master_edition_info)?;
    let printing_mint = token_unpack::<Mint>(&printing_mint_info.try_borrow_data()?)?.base;
    let auth_mint =
        token_unpack::<Mint>(&one_time_printing_auth_mint_info.try_borrow_data()?)?.base;
    if master_edition.one_time_printing_authorization_mint != *one_time_printing_auth_mint_info.key
    {
        return Err(MetadataError::OneTimePrintingAuthMintMismatch.into());
    }

    if master_edition.printing_mint != *printing_mint_info.key {
        return Err(MetadataError::PrintingMintMismatch.into());
    }

    if printing_mint.supply != 0 {
        return Err(MetadataError::PrintingMintSupplyMustBeZeroForConversion.into());
    }

    if auth_mint.supply != 0 {
        return Err(MetadataError::OneTimeAuthMintSupplyMustBeZeroForConversion.into());
    }

    MasterEditionV2 {
        key: Key::MasterEditionV2,
        supply: master_edition.supply,
        max_supply: master_edition.max_supply,
    }
    .serialize(&mut *master_edition_info.try_borrow_mut_data()?)?;

    Ok(())
}
