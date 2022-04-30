use crate::{CandyError, CandyMachine};
use anchor_lang::prelude::*;
use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    program::invoke_signed,
    program_memory::sol_memcmp,
    program_pack::{IsInitialized, Pack},
    pubkey::{Pubkey, PUBKEY_BYTES},
};
use spl_associated_token_account::get_associated_token_address;

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
    clock: &Sysvar<Clock>,
    candy_machine: &Account<'info, CandyMachine>,
) -> Result<()> {
    match candy_machine.data.go_live_date {
        None => {
            if *payer.key != candy_machine.authority {
                return Err(CandyError::CandyMachineNotLive.into());
            }
        }
        Some(val) => {
            if clock.unix_timestamp < val && *payer.key != candy_machine.authority {
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
    assert_keys_equal(ata_account.owner, *wallet)?;
    assert_keys_equal(ata_account.mint, *mint)?;
    assert_keys_equal(get_associated_token_address(wallet, mint), *ata.key)?;
    Ok(ata_account)
}

pub fn assert_keys_equal(key1: Pubkey, key2: Pubkey) -> Result<()> {
    if !cmp_pubkeys(&key1, &key2) {
        Err(CandyError::PublicKeyMismatch.into())
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
// feature_index is between 0 and 5, inclusive
// unsafe is fine because we know for a fact that the vec will only
// contain valid UTF8 bytes (we set it as "1" or "0")
pub fn set_feature_flag(uuid: &str, feature_index: usize) -> String {
    let mut bytes: Vec<u8> = vec![b"0"[0]; 6];
    uuid.bytes().enumerate().for_each(|(i, byte)| {
        if i == feature_index || byte == "1".as_bytes()[0] {
            bytes[i] = b"1"[0]
        }
    });
    unsafe { String::from_utf8_unchecked(bytes) }
}

pub fn remove_feature_flag(uuid: &str, feature_index: usize) -> String {
    let mut bytes: Vec<u8> = vec![b"0"[0]; 6];
    uuid.bytes().enumerate().for_each(|(i, byte)| {
        if i == feature_index {
            bytes[i] = b"0"[0];
        } else if byte == "1".as_bytes()[0] {
            bytes[i] = b"1"[0];
        }
    });
    unsafe { String::from_utf8_unchecked(bytes) }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::candy_machine::COLLECTIONS_FEATURE_INDEX;

    #[test]
    fn feature_flag_working() {
        let mut uuid = String::from("ABCDEF");
        println!("{}", uuid.as_bytes()[COLLECTIONS_FEATURE_INDEX]);
        assert!(!is_feature_active(&uuid, COLLECTIONS_FEATURE_INDEX));

        uuid = set_feature_flag(&uuid, COLLECTIONS_FEATURE_INDEX);
        println!("{}", uuid);
        assert!(is_feature_active(&uuid, COLLECTIONS_FEATURE_INDEX));
        uuid = remove_feature_flag(&uuid, COLLECTIONS_FEATURE_INDEX);
        println!("{}", uuid);
        assert!(!is_feature_active(&uuid, COLLECTIONS_FEATURE_INDEX));

        let uuid = String::from("01H333");
        println!("{}", uuid);
        assert!(!is_feature_active(&uuid, COLLECTIONS_FEATURE_INDEX));
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
        assert_eq!(cmp_pubkeys(&key1, &key2), false);
    }
}
