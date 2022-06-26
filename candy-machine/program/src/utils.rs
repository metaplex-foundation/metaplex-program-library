use std::str::from_utf8_unchecked;

use anchor_lang::prelude::*;
use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    program::{invoke, invoke_signed},
    program_memory::sol_memcmp,
    program_pack::{IsInitialized, Pack},
    pubkey::{Pubkey, PUBKEY_BYTES},
    system_instruction,
};
use spl_associated_token_account::get_associated_token_address;

use crate::{CandyError, CandyMachine};

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

pub fn assert_valid_go_live<'info>(
    payer: &Signer<'info>,
    clock: Clock,
    candy_machine: &Account<'info, CandyMachine>,
) -> Result<()> {
    match candy_machine.data.go_live_date {
        None => {
            if !cmp_pubkeys(payer.key, &candy_machine.authority) {
                return Err(CandyError::CandyMachineNotLive.into());
            }
        }
        Some(val) => {
            if clock.unix_timestamp < val && !cmp_pubkeys(payer.key, &candy_machine.authority) {
                return Err(CandyError::CandyMachineNotLive.into());
            }
        }
    }

    Ok(())
}

pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> Result<()> {
    if !cmp_pubkeys(account.owner, owner) {
        Err(CandyError::IncorrectOwner.into())
    } else {
        Ok(())
    }
}
///TokenTransferParams
pub struct TokenTransferParams<'a: 'b, 'b> {
    /// source
    /// CHECK: account checked in CPI
    pub source: AccountInfo<'a>,
    /// destination
    /// CHECK: account checked in CPI
    pub destination: AccountInfo<'a>,
    /// amount
    pub amount: u64,
    /// authority
    /// CHECK: account checked in CPI
    pub authority: AccountInfo<'a>,
    /// authority_signer_seeds
    pub authority_signer_seeds: &'b [&'b [u8]],
    /// token_program
    /// CHECK: account checked in CPI
    pub token_program: AccountInfo<'a>,
}

#[inline(always)]
pub fn spl_token_transfer(params: TokenTransferParams<'_, '_>) -> Result<()> {
    let TokenTransferParams {
        source,
        destination,
        authority,
        token_program,
        amount,
        authority_signer_seeds,
    } = params;

    let mut signer_seeds = vec![];
    if !authority_signer_seeds.is_empty() {
        signer_seeds.push(authority_signer_seeds)
    }

    let result = invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?,
        &[source, destination, authority, token_program],
        &signer_seeds,
    );

    result.map_err(|_| CandyError::TokenTransferFailed.into())
}

pub fn assert_is_ata(
    ata: &AccountInfo,
    wallet: &Pubkey,
    mint: &Pubkey,
) -> core::result::Result<spl_token::state::Account, ProgramError> {
    assert_owned_by(ata, &spl_token::id())?;
    let ata_account: spl_token::state::Account = assert_initialized(ata)?;
    assert_keys_equal(&ata_account.owner, wallet)?;
    assert_keys_equal(&ata_account.mint, mint)?;
    assert_keys_equal(&get_associated_token_address(wallet, mint), ata.key)?;
    Ok(ata_account)
}

pub fn assert_keys_equal(key1: &Pubkey, key2: &Pubkey) -> Result<()> {
    if !cmp_pubkeys(key1, key2) {
        err!(CandyError::PublicKeyMismatch)
    } else {
        Ok(())
    }
}

/// TokenBurnParams
pub struct TokenBurnParams<'a: 'b, 'b> {
    /// mint
    /// CHECK: account checked in CPI
    pub mint: AccountInfo<'a>,
    /// source
    /// CHECK: account checked in CPI
    pub source: AccountInfo<'a>,
    /// amount
    pub amount: u64,
    /// authority
    /// CHECK: account checked in CPI
    pub authority: AccountInfo<'a>,
    /// authority_signer_seeds
    pub authority_signer_seeds: Option<&'b [&'b [u8]]>,
    /// token_program
    /// CHECK: account checked in CPI
    pub token_program: AccountInfo<'a>,
}

pub fn spl_token_burn(params: TokenBurnParams<'_, '_>) -> Result<()> {
    let TokenBurnParams {
        mint,
        source,
        authority,
        token_program,
        amount,
        authority_signer_seeds,
    } = params;
    let mut seeds: Vec<&[&[u8]]> = vec![];
    if let Some(seed) = authority_signer_seeds {
        seeds.push(seed);
    }
    let result = invoke_signed(
        &spl_token::instruction::burn(
            token_program.key,
            source.key,
            mint.key,
            authority.key,
            &[],
            amount,
        )?,
        &[source, mint, authority, token_program],
        seeds.as_slice(),
    );
    result.map_err(|_| CandyError::TokenBurnFailed.into())
}

pub fn is_feature_active(uuid: &str, feature_index: usize) -> bool {
    uuid.as_bytes()[feature_index] == b"1"[0]
}

// string is 6 bytes long, can be any valid utf8 char coming in.
// feature_index is between 0 and 5, inclusive. We set it to an array of utf8 "0"s first
pub fn set_feature_flag(uuid: &mut String, feature_index: usize) {
    let mut bytes: [u8; 6] = [b'0'; 6];
    uuid.bytes().enumerate().for_each(|(i, byte)| {
        if i == feature_index || byte == b'1' {
            bytes[i] = b'1';
        }
    });

    // unsafe is fine because we know for a fact that the array will only
    // contain valid UTF8 bytes since we fully ignore user inputted UUID and set
    // it to an array of only valid bytes (b'0') and then only modify the bytes in
    // that valid utf8 byte array to other valid utf8 characters (b'1')
    // This saves a bit of compute from the overhead of using the from_utf8 or
    // other similar methods that need to ensure that the bytes are valid
    unsafe {
        uuid.replace_range(.., from_utf8_unchecked(&bytes));
    }
}

// string is 6 bytes long, can be any valid utf8 char coming in.
// feature_index is between 0 and 5, inclusive. We set it to an array of utf8 "0"s first
pub fn remove_feature_flag(uuid: &mut String, feature_index: usize) {
    let mut bytes: [u8; 6] = [b'0'; 6];
    uuid.bytes().enumerate().for_each(|(i, byte)| {
        if i == feature_index {
            bytes[i] = b'0';
        } else if byte == b'1' {
            bytes[i] = b'1';
        }
    });

    // unsafe is fine because we know for a fact that the array will only
    // contain valid UTF8 bytes since we fully ignore user inputted UUID and set
    // it to an array of only valid bytes (b'0') and then only modify the bytes in
    // that valid utf8 byte array to other valid utf8 characters (b'1')
    // This saves a bit of compute from the overhead of using the from_utf8 or
    // other similar methods that need to ensure that the bytes are valid
    unsafe {
        uuid.replace_range(.., from_utf8_unchecked(&bytes));
    }
}

pub fn punish_bots<'a>(
    err: CandyError,
    bot_account: AccountInfo<'a>,
    payment_account: AccountInfo<'a>,
    system_program: AccountInfo<'a>,
    fee: u64,
) -> Result<()> {
    msg!(
        "{}, Candy Machine Botting is taxed at {:?} lamports",
        err.to_string(),
        fee
    );
    let final_fee = fee.min(bot_account.lamports());
    invoke(
        &system_instruction::transfer(bot_account.key, payment_account.key, final_fee),
        &[bot_account, payment_account, system_program],
    )?;
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::constants::COLLECTIONS_FEATURE_INDEX;

    #[test]
    fn feature_flag_working() {
        let mut uuid = String::from("ABCDEF");
        println!(
            "Should be 65: {}",
            uuid.as_bytes()[COLLECTIONS_FEATURE_INDEX]
        );

        uuid = String::from("01H333");
        println!("Should be 01H333: {}", uuid);
        set_feature_flag(&mut uuid, COLLECTIONS_FEATURE_INDEX + 1);
        assert!(is_feature_active(&uuid, COLLECTIONS_FEATURE_INDEX + 1));
        println!("Should be 010000: {}", uuid);
        remove_feature_flag(&mut uuid, COLLECTIONS_FEATURE_INDEX + 1);
        assert!(!is_feature_active(&uuid, COLLECTIONS_FEATURE_INDEX + 1));
        println!("Should be 000000: {}", uuid);

        set_feature_flag(&mut uuid, COLLECTIONS_FEATURE_INDEX);
        assert!(is_feature_active(&uuid, COLLECTIONS_FEATURE_INDEX));
        println!("Should be 100000: {}", uuid);
        remove_feature_flag(&mut uuid, COLLECTIONS_FEATURE_INDEX);
        assert!(!is_feature_active(&uuid, COLLECTIONS_FEATURE_INDEX));
        println!("Should be 000000: {}", uuid);
    }

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
