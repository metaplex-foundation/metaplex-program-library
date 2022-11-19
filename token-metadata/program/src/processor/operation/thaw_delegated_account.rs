use borsh::BorshSerialize;
use mpl_utils::assert_signer;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
    pubkey::Pubkey,
};
use spl_token::{instruction::thaw_account, state::Mint};

use crate::{
    assertions::{
        assert_delegated_tokens, assert_derivation, assert_freeze_authority_matches_mint,
        assert_initialized, assert_owned_by,
    },
    error::MetadataError,
    instruction::MetadataInstruction,
    state::{EDITION, PREFIX},
};

pub(crate) mod instruction {
    use super::*;

    ///# Thaw delegated account
    ///
    ///Allow thawing of an NFT if this user is the delegate of the NFT
    ///
    ///### Accounts:
    ///   0. `[signer]` Delegate
    ///   1. `[writable]` Token account to thaw
    ///   2. `[]` Edition
    ///   3. `[]` Token mint
    #[allow(clippy::too_many_arguments)]
    pub fn thaw_delegated_account(
        program_id: Pubkey,
        delegate: Pubkey,
        token_account: Pubkey,
        edition: Pubkey,
        mint: Pubkey,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(delegate, true),
                AccountMeta::new(token_account, false),
                AccountMeta::new_readonly(edition, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
            data: MetadataInstruction::ThawDelegatedAccount
                .try_to_vec()
                .unwrap(),
        }
    }
}

pub fn thaw_delegated_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let delegate_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let edition_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let token_program_account_info = next_account_info(account_info_iter)?;
    if *token_program_account_info.key != spl_token::id() {
        return Err(MetadataError::InvalidTokenProgram.into());
    }

    // assert that edition pda is the freeze authority of this mint
    let mint: Mint = assert_initialized(mint_info)?;
    assert_owned_by(edition_info, program_id)?;
    assert_freeze_authority_matches_mint(&mint.freeze_authority, edition_info)?;

    // assert delegate is signer and delegated tokens
    assert_signer(delegate_info)?;
    assert_delegated_tokens(delegate_info, mint_info, token_account_info)?;

    let edition_info_path = Vec::from([
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint_info.key.as_ref(),
        EDITION.as_bytes(),
    ]);
    let edition_info_path_bump_seed = &[assert_derivation(
        program_id,
        edition_info,
        &edition_info_path,
    )?];
    let mut edition_info_seeds = edition_info_path.clone();
    edition_info_seeds.push(edition_info_path_bump_seed);
    invoke_signed(
        &thaw_account(
            token_program_account_info.key,
            token_account_info.key,
            mint_info.key,
            edition_info.key,
            &[],
        )
        .unwrap(),
        &[
            token_account_info.clone(),
            mint_info.clone(),
            edition_info.clone(),
        ],
        &[&edition_info_seeds],
    )?;
    Ok(())
}
