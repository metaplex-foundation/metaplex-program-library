use borsh::BorshSerialize;
pub use instruction::*;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::{
    assertions::{
        assert_derivation, assert_owned_by, metadata::assert_update_authority_is_correct,
    },
    deser::clean_write_metadata,
    error::MetadataError,
    instruction::MetadataInstruction,
    state::{Metadata, TokenMetadataAccount, EDITION, PREFIX},
    utils::check_token_standard,
};

mod instruction {
    use super::*;

    pub fn set_token_standard(
        program_id: Pubkey,
        metadata_account: Pubkey,
        update_authority: Pubkey,
        mint_account: Pubkey,
        edition_account: Option<Pubkey>,
    ) -> Instruction {
        let mut accounts = vec![
            AccountMeta::new(metadata_account, false),
            AccountMeta::new(update_authority, true),
            AccountMeta::new_readonly(mint_account, false),
        ];
        let data = MetadataInstruction::SetTokenStandard.try_to_vec().unwrap();

        if let Some(edition_account) = edition_account {
            accounts.push(AccountMeta::new_readonly(edition_account, false));
        }

        Instruction {
            program_id,
            accounts,
            data,
        }
    }
}

pub fn process_set_token_standard(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let metadata_account_info = next_account_info(account_info_iter)?;
    let update_authority_account_info = next_account_info(account_info_iter)?;
    let mint_account_info = next_account_info(account_info_iter)?;

    // Owned by token-metadata program.
    assert_owned_by(metadata_account_info, program_id)?;
    let mut metadata = Metadata::from_account_info(metadata_account_info)?;

    // Mint account passed in must be the mint of the metadata account passed in.
    if &metadata.mint != mint_account_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    // Update authority is a signer and matches update authority on metadata.
    assert_update_authority_is_correct(&metadata, update_authority_account_info)?;

    // Edition account provided.
    let token_standard = if accounts.len() == 4 {
        let edition_account_info = next_account_info(account_info_iter)?;

        let edition_path = Vec::from([
            PREFIX.as_bytes(),
            program_id.as_ref(),
            mint_account_info.key.as_ref(),
            EDITION.as_bytes(),
        ]);
        assert_owned_by(edition_account_info, program_id)?;
        assert_derivation(program_id, edition_account_info, &edition_path)?;

        check_token_standard(mint_account_info, Some(edition_account_info))?
    } else {
        check_token_standard(mint_account_info, None)?
    };

    metadata.token_standard = Some(token_standard);
    clean_write_metadata(&mut metadata, metadata_account_info)?;
    Ok(())
}
