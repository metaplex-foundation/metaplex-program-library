use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError, pubkey,
    pubkey::Pubkey,
};
use spl_token_2022::state::Account;

use crate::assert_initialized;

pub static SPL_TOKEN_PROGRAM_IDS: [Pubkey; 2] = [
    pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
    pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"),
];

pub trait ToTokenAccount {
    fn to_token_account(self) -> Account;
}

impl ToTokenAccount for AccountInfo<'_> {
    fn to_token_account(self) -> Account {
        assert_initialized(&self, ProgramError::UninitializedAccount).unwrap()
    }
}

impl ToTokenAccount for Account {
    fn to_token_account(self) -> Account {
        self
    }
}

pub fn assert_token_program_matches_package(
    token_program_info: &AccountInfo,
    error: impl Into<ProgramError>,
) -> ProgramResult {
    if SPL_TOKEN_PROGRAM_IDS
        .iter()
        .any(|program_id| program_id == token_program_info.key)
    {
        Ok(())
    } else {
        Err(error.into())
    }
}

/// Asserts that
/// * the given token account is initialized
/// * it's owner matches the provided owner
/// * it's mint matches the provided mint
/// * it holds more than than 0 tokens of the given mint.
/// Accepts either an &AccountInfo or an Account for token_account parameter.
pub fn assert_holder(
    token_account: impl ToTokenAccount,
    owner_info: &AccountInfo,
    mint_info: &AccountInfo,
    error: impl Into<ProgramError> + Clone,
) -> ProgramResult {
    let token_account: Account = token_account.to_token_account();

    if token_account.owner != *owner_info.key {
        return Err(error.into());
    }

    if token_account.mint != *mint_info.key {
        return Err(error.into());
    }

    if token_account.amount == 0 {
        return Err(error.into());
    }

    Ok(())
}
