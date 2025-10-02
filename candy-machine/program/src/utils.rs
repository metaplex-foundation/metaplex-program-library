use anchor_lang::prelude::*;
use mpl_token_metadata::{
    error::MetadataError,
    state::{MasterEditionV2, Metadata, TokenStandard},
};
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

use crate::{constants::*, CandyError, CandyMachine};

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
    clock: &Clock,
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
    let uuid_bytes = uuid.as_bytes();
    if feature_index == COLLECTIONS_FEATURE_INDEX && uuid_bytes[feature_index] == b'1' {
        is_valid_uuid(uuid)
    } else {
        uuid_bytes[feature_index] == b'#'
    }
}

fn is_valid_uuid(uuid: &str) -> bool {
    !uuid.bytes().any(|b| b != b'1' && b != b'0' && b != b'#')
}

// string is 6 bytes long, can be any valid utf8 char coming in.
// feature_index is between 0 and 5, inclusive.
pub fn set_feature_flag(uuid: &mut str, feature_index: usize) {
    if feature_index > 5 {
        return;
    }

    // its safe because the char boundaries for the normalized string are all 1 byte. trust me bro
    unsafe {
        uuid.as_bytes_mut()[feature_index] = b'#';
    }
}

// string is 6 bytes long, can be any valid utf8 char coming in.
// feature_index is between 0 and 5, inclusive.
pub fn remove_feature_flag(uuid: &mut str, feature_index: usize) {
    if feature_index > 5 {
        return;
    }

    // its safe because the char boundaries for the normalized string are all 1 byte. trust me bro
    unsafe {
        uuid.as_bytes_mut()[feature_index] = b'0';
    }
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

pub fn assert_master_edition(
    collection_data: &Metadata,
    edition_account_info: &AccountInfo,
) -> core::result::Result<(), ProgramError> {
    let data = edition_account_info.try_borrow_data()?;
    if data.is_empty() || data[0] != mpl_token_metadata::state::Key::MasterEditionV2 as u8 {
        return Err(MetadataError::DataTypeMismatch.into());
    }
    let edition = MasterEditionV2::deserialize(&mut data.as_ref())
        .map_err(|_err: std::io::Error| MetadataError::CollectionMustBeAUniqueMasterEdition)?;

    match collection_data.token_standard {
        Some(TokenStandard::NonFungible) | Some(TokenStandard::ProgrammableNonFungible) => (),
        _ => return Err(MetadataError::CollectionMustBeAUniqueMasterEdition.into()),
    }

    if edition.max_supply != Some(0) {
        return Err(MetadataError::CollectionMustBeAUniqueMasterEdition.into());
    }
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use std::{assert_eq, println};

    use crate::constants::COLLECTIONS_FEATURE_INDEX;

    use super::*;

    #[test]
    fn feature_flag_working() {
        let mut uuid = String::from("ABCDEF");
        println!(
            "Should be 65: {}",
            uuid.as_bytes()[COLLECTIONS_FEATURE_INDEX]
        );
        remove_feature_flag(&mut uuid, COLLECTIONS_FEATURE_INDEX);
        assert_eq!(uuid, "0BCDEF");

        uuid = String::from("01H333");
        assert!(!is_feature_active(&uuid, FREEZE_FEATURE_INDEX));
        assert_eq!(uuid, "01H333");
        set_feature_flag(&mut uuid, FREEZE_FEATURE_INDEX);
        assert_eq!(uuid, "0#H333");
        assert!(is_feature_active(&uuid, FREEZE_FEATURE_INDEX));

        remove_feature_flag(&mut uuid, FREEZE_FEATURE_INDEX);
        assert!(!is_feature_active(&uuid, FREEZE_FEATURE_INDEX));
        println!("Should be 00H333: {}", uuid);

        set_feature_flag(&mut uuid, FREEZE_LOCK_FEATURE_INDEX);
        assert!(is_feature_active(&uuid, FREEZE_LOCK_FEATURE_INDEX));
        assert_eq!(uuid, "00#333");

        remove_feature_flag(&mut uuid, COLLECTIONS_FEATURE_INDEX);
        assert_eq!(uuid, "00#333");
        set_feature_flag(&mut uuid, FREEZE_FEATURE_INDEX);
        assert!(is_feature_active(&uuid, FREEZE_FEATURE_INDEX));
        assert_eq!(uuid, "0##333");
        set_feature_flag(&mut uuid, COLLECTIONS_FEATURE_INDEX);
        assert!(is_feature_active(&uuid, COLLECTIONS_FEATURE_INDEX));
        assert_eq!(uuid, "###333");

        uuid = String::from("1ABCDE");
        assert!(!is_feature_active(&uuid, COLLECTIONS_FEATURE_INDEX));
        uuid = String::from("100000");
        assert!(is_feature_active(&uuid, COLLECTIONS_FEATURE_INDEX));

        uuid = String::from("1##000");
        assert!(is_feature_active(&uuid, COLLECTIONS_FEATURE_INDEX));
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
