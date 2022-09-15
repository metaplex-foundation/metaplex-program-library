use anchor_lang::prelude::*;
use solana_program::{
    account_info::AccountInfo,
    program::invoke,
    program_memory::sol_memcmp,
    program_pack::{IsInitialized, Pack},
    pubkey::{Pubkey, PUBKEY_BYTES},
    system_instruction,
};

use crate::{
    constants::{NULL_STRING, REPLACEMENT_INDEX, REPLACEMENT_INDEX_INCREMENT},
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
