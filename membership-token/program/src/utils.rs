//! Module provide runtime utilities

use crate::{id, ErrorCode};
use anchor_lang::prelude::*;

pub const NAME_MAX_LEN: usize = 40; // max len of a string buffer in bytes
pub const NAME_DEFAULT_SIZE: usize = 4 + NAME_MAX_LEN; // max lenght of serialized string (str_len + <buffer>)
pub const DESCRIPTION_MAX_LEN: usize = 60;
pub const DESCRIPTION_DEFAULT_SIZE: usize = 4 + DESCRIPTION_MAX_LEN;
pub const HOLDER_PREFIX: &str = "holder";
pub const HISTORY_PREFIX: &str = "history";
pub const VAULT_OWNER_PREFIX: &str = "mt_vault";

/// Runtime derivation check
pub fn assert_derivation(
    program_id: &Pubkey,
    account: &AccountInfo,
    path: &[&[u8]],
) -> Result<u8, ProgramError> {
    let (key, bump) = Pubkey::find_program_address(&path, program_id);
    if key != *account.key {
        return Err(ErrorCode::DerivedKeyInvalid.into());
    }
    Ok(bump)
}

/// Return `treasury_owner` Pubkey and bump seed.
pub fn find_treasury_owner_address(
    treasury_mint: &Pubkey,
    selling_resource: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            HOLDER_PREFIX.as_bytes(),
            treasury_mint.as_ref(),
            selling_resource.as_ref(),
        ],
        &id(),
    )
}

/// Return `vault_owner` Pubkey and bump seed.
pub fn find_vault_owner_address(resource_mint: &Pubkey, store: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            VAULT_OWNER_PREFIX.as_bytes(),
            resource_mint.as_ref(),
            store.as_ref(),
        ],
        &id(),
    )
}

/// Return `TradeHistory` Pubkey and bump seed.
pub fn find_trade_history_address(wallet: &Pubkey, market: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[HISTORY_PREFIX.as_bytes(), wallet.as_ref(), market.as_ref()],
        &id(),
    )
}

pub fn puffed_out_string(s: &String, size: usize) -> String {
    let mut array_of_zeroes = vec![];
    let puff_amount = size - s.len();
    while array_of_zeroes.len() < puff_amount {
        array_of_zeroes.push(0u8);
    }
    s.clone() + std::str::from_utf8(&array_of_zeroes).unwrap()
}

/// Two keys equivalence check
pub fn assert_keys_equal(key1: Pubkey, key2: Pubkey) -> ProgramResult {
    if key1 != key2 {
        Err(ErrorCode::PublicKeyMismatch.into())
    } else {
        Ok(())
    }
}
