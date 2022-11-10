use anchor_lang::prelude::*;
use arrayref::array_ref;
use mpl_token_metadata::{
    error::MetadataError,
    state::{EDITION, PREFIX},
    utils::assert_derivation,
};
use solana_program::{
    account_info::AccountInfo,
    program::invoke,
    program_memory::sol_memcmp,
    program_pack::{IsInitialized, Pack},
    pubkey::{Pubkey, PUBKEY_BYTES},
    system_instruction,
};
use std::result::Result as StdResult;

use crate::{
    constants::{HIDDEN_SECTION, NULL_STRING, REPLACEMENT_INDEX, REPLACEMENT_INDEX_INCREMENT},
    CandyError,
};

pub fn assert_initialized<T: Pack + IsInitialized>(account_info: &AccountInfo) -> Result<T> {
    let account: T = T::unpack_unchecked(&account_info.data.borrow())?;
    if !account.is_initialized() {
        Err(CandyError::Uninitialized.into())
    } else {
        Ok(account)
    }
}

/// Return the current number of lines written to the account.
pub fn get_config_count(data: &[u8]) -> Result<usize> {
    Ok(u32::from_le_bytes(*array_ref![data, HIDDEN_SECTION, 4]) as usize)
}

pub fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
    sol_memcmp(a.as_ref(), b.as_ref(), PUBKEY_BYTES) == 0
}

/// Return a padded string up to the specified length. If the specified
/// string `value` is longer than the allowed `length`, return an error.
pub fn fixed_length_string(value: String, length: usize) -> Result<String> {
    if length < value.len() {
        // the value is larger than the allowed length
        return err!(CandyError::ExceededLengthError);
    }

    let padding = NULL_STRING.repeat(length - value.len());
    Ok(value + &padding)
}

pub fn punish_bots<'a>(
    error: CandyError,
    bot_account: AccountInfo<'a>,
    payment_account: AccountInfo<'a>,
    system_program: AccountInfo<'a>,
    fee: u64,
) -> Result<()> {
    msg!(
        "{}, Candy Machine Botting is taxed at {:?} lamports",
        error.to_string(),
        fee
    );

    let final_fee = fee.min(bot_account.lamports());
    invoke(
        &system_instruction::transfer(bot_account.key, payment_account.key, final_fee),
        &[bot_account, payment_account, system_program],
    )?;
    Ok(())
}

/// Replace the index pattern variables on the specified string.
pub fn replace_patterns(value: String, index: usize) -> String {
    let mut mutable = value;
    // check for pattern $ID+1$
    if mutable.contains(REPLACEMENT_INDEX_INCREMENT) {
        mutable = mutable.replace(REPLACEMENT_INDEX_INCREMENT, &(index + 1).to_string());
    }
    // check for pattern $ID$
    if mutable.contains(REPLACEMENT_INDEX) {
        mutable = mutable.replace(REPLACEMENT_INDEX, &index.to_string());
    }

    mutable
}

pub fn assert_edition_from_mint(
    edition_account: &AccountInfo,
    mint_account: &AccountInfo,
) -> StdResult<(), ProgramError> {
    assert_derivation(
        &mpl_token_metadata::id(),
        edition_account,
        &[
            PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            mint_account.key().as_ref(),
            EDITION.as_bytes(),
        ],
    )
    .map_err(|_| MetadataError::CollectionMasterEditionAccountInvalid)?;
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn check_keys_equal() {
        let key1 = Pubkey::new_unique();
        assert!(cmp_pubkeys(&key1, &key1));
    }

    #[test]
    fn check_keys_not_equal() {
        let key1 = Pubkey::new_unique();
        let key2 = Pubkey::new_unique();
        assert!(!cmp_pubkeys(&key1, &key2));
    }
}
