use anchor_lang::prelude::*;
use spl_token_metadata::instruction::mint_new_edition_from_master_edition_via_token;
use std::slice::Iter;

use crate::errors;

pub fn mint_new_edition_cpi<'info>(
  account_iter: &mut Iter<AccountInfo<'info>>,
  payer: &AccountInfo<'info>,
  system_program: &AccountInfo<'info>,
  rent_acct: &AccountInfo<'info>,
  signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
  let metadata_program_acct = next_account_info(account_iter)?;
  let new_metadata_acct = next_account_info(account_iter)?;
  let new_edition_acct = next_account_info(account_iter)?;
  let master_edition_acct = next_account_info(account_iter)?;
  let new_mint_acct = next_account_info(account_iter)?;
  let edition_mark_pda_acct = next_account_info(account_iter)?;
  let new_mint_authority_acct = next_account_info(account_iter)?;
  let token_account_owner_acct = next_account_info(account_iter)?;
  let token_account_acct = next_account_info(account_iter)?;
  let new_metadata_update_authority_acct = next_account_info(account_iter)?;
  let metadata_acct = next_account_info(account_iter)?;
  let metadata_mint_acct = next_account_info(account_iter)?;

  // check that the metadata program is expected
  if !spl_token_metadata::check_id(metadata_program_acct.key) {
    return Err(errors::ErrorCode::InvalidTokenMetadataProgram.into())
  }

  let ix = mint_new_edition_from_master_edition_via_token(
    *metadata_program_acct.key,
    *new_metadata_acct.key,
    *new_edition_acct.key,
    *master_edition_acct.key,
    *new_mint_acct.key,
    *new_mint_authority_acct.key,
    *payer.key,
    *token_account_owner_acct.key,
    *token_account_acct.key,
    *new_metadata_update_authority_acct.key,
    *metadata_acct.key,
    *metadata_mint_acct.key,
    1,
  );
  // send mint from master edition token
  anchor_lang::solana_program::program::invoke_signed(
    &ix,
    &[
      metadata_program_acct.clone(),
      new_metadata_acct.clone(),
      new_edition_acct.clone(),
      master_edition_acct.clone(),
      new_mint_acct.clone(),
      edition_mark_pda_acct.clone(),
      new_mint_authority_acct.clone(),
      token_account_owner_acct.clone(),
      token_account_acct.clone(),
      new_metadata_update_authority_acct.clone(),
      metadata_acct.clone(),
      metadata_mint_acct.clone(),
      system_program.clone(),
      rent_acct.clone(),
      payer.clone(),
    ],
    signers_seeds,
  )?;

  Ok(())
}
