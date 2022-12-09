use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed,
};
use spl_token::instruction::{freeze_account, thaw_account};

use crate::{
    assertions::assert_derivation,
    pda::{EDITION, PREFIX},
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
