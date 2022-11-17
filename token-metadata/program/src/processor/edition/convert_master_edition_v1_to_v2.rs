use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use spl_token::state::Mint;

use crate::{
    assertions::{assert_initialized, assert_owned_by},
    error::MetadataError,
    instruction_old::MetadataInstruction,
    state::{Key, MasterEditionV1, MasterEditionV2, TokenMetadataAccount},
};

pub(crate) mod instruction {
    use super::*;

    /// Converts a master edition v1 to v2
    #[allow(clippy::too_many_arguments)]
    pub fn convert_master_edition_v1_to_v2(
        program_id: Pubkey,
        master_edition: Pubkey,
        one_time_auth: Pubkey,
        printing_mint: Pubkey,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(master_edition, false),
                AccountMeta::new(one_time_auth, false),
                AccountMeta::new(printing_mint, false),
            ],
            data: MetadataInstruction::ConvertMasterEditionV1ToV2
                .try_to_vec()
                .unwrap(),
        }
    }
}

pub fn process_convert_master_edition_v1_to_v2(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let master_edition_info = next_account_info(account_info_iter)?;
    let one_time_printing_auth_mint_info = next_account_info(account_info_iter)?;
    let printing_mint_info = next_account_info(account_info_iter)?;

    assert_owned_by(master_edition_info, program_id)?;
    assert_owned_by(one_time_printing_auth_mint_info, &spl_token::id())?;
    assert_owned_by(printing_mint_info, &spl_token::id())?;
    let master_edition = MasterEditionV1::from_account_info(master_edition_info)?;
    let printing_mint: Mint = assert_initialized(printing_mint_info)?;
    let auth_mint: Mint = assert_initialized(one_time_printing_auth_mint_info)?;
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
