use mpl_token_auth_rules::state::Operation;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed,
};
use spl_token::instruction::{freeze_account, thaw_account};

use crate::{
    assertions::assert_derivation,
    pda::{EDITION, PREFIX},
    processor::AuthorizationData,
};

pub fn freeze<'a>(
    mint: &'a AccountInfo<'a>,
    token: &'a AccountInfo<'a>,
    edition: &'a AccountInfo<'a>,
    spl_token_program: &'a AccountInfo<'a>,
) -> ProgramResult {
    let edition_info_path = Vec::from([
        PREFIX.as_bytes(),
        crate::ID.as_ref(),
        mint.key.as_ref(),
        EDITION.as_bytes(),
    ]);
    let edition_info_path_bump_seed = &[assert_derivation(
        &crate::id(),
        edition,
        &edition_info_path,
    )?];
    let mut edition_info_seeds = edition_info_path.clone();
    edition_info_seeds.push(edition_info_path_bump_seed);

    invoke_signed(
        &freeze_account(spl_token_program.key, token.key, mint.key, edition.key, &[]).unwrap(),
        &[token.clone(), mint.clone(), edition.clone()],
        &[&edition_info_seeds],
    )?;
    Ok(())
}

pub fn thaw<'a>(
    mint: &'a AccountInfo<'a>,
    token: &'a AccountInfo<'a>,
    edition: &'a AccountInfo<'a>,
    spl_token_program: &'a AccountInfo<'a>,
) -> ProgramResult {
    let edition_info_path = Vec::from([
        PREFIX.as_bytes(),
        crate::ID.as_ref(),
        mint.key.as_ref(),
        EDITION.as_bytes(),
    ]);
    let edition_info_path_bump_seed = &[assert_derivation(
        &crate::id(),
        edition,
        &edition_info_path,
    )?];
    let mut edition_info_seeds = edition_info_path.clone();
    edition_info_seeds.push(edition_info_path_bump_seed);

    invoke_signed(
        &thaw_account(spl_token_program.key, token.key, mint.key, edition.key, &[]).unwrap(),
        &[token.clone(), mint.clone(), edition.clone()],
        &[&edition_info_seeds],
    )?;
    Ok(())
}

pub fn validate<'a>(
    payer: &'a AccountInfo<'a>,
    ruleset: &'a AccountInfo<'a>,
    destination_owner: &'a AccountInfo<'a>,
    auth_data: &AuthorizationData,
) {
    let validate_ix = mpl_token_auth_rules::instruction::validate(
        mpl_token_auth_rules::ID,
        *payer.key,
        *ruleset.key,
        auth_data.name.clone(),
        Operation::Transfer,
        auth_data.payload.clone(),
        vec![],
        vec![*destination_owner.key],
    );

    invoke_signed(
        &validate_ix,
        &[payer.clone(), ruleset.clone(), destination_owner.clone()],
        &[],
    )
    .unwrap();
}
