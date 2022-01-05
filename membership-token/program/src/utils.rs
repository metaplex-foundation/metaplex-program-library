//! Module provide runtime utilities

use crate::{id, ErrorCode};
use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke_signed, system_instruction},
};

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
    let (key, bump) = Pubkey::find_program_address(path, program_id);
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

/// Wrapper of `create_account` instruction from `system_program` program
#[inline(always)]
pub fn sys_create_account<'a>(
    from: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    lamports: u64,
    space: usize,
    owner: &Pubkey,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    invoke_signed(
        &system_instruction::create_account(from.key, to.key, lamports, space as u64, owner),
        &[from.clone(), to.clone()],
        &[&signer_seeds],
    )?;

    Ok(())
}

/// Wrapper of `mint_new_edition_from_master_edition_via_token` instruction from `mpl_token_metadata` program
#[inline(always)]
pub fn mpl_mint_new_edition_from_master_edition_via_token<'a>(
    new_metadata: &AccountInfo<'a>,
    new_edition: &AccountInfo<'a>,
    new_mint: &AccountInfo<'a>,
    new_mint_authority: &AccountInfo<'a>,
    user_wallet: &AccountInfo<'a>,
    token_account_owner: &AccountInfo<'a>,
    token_account: &AccountInfo<'a>,
    master_metadata: &AccountInfo<'a>,
    master_edition: &AccountInfo<'a>,
    metadata_mint: &Pubkey,
    edition_marker: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
    edition: u64,
    signers_seeds: &[&[u8]],
) -> ProgramResult {
    let tx = mpl_token_metadata::instruction::mint_new_edition_from_master_edition_via_token(
        mpl_token_metadata::id(),
        *new_metadata.key,
        *new_edition.key,
        *master_edition.key,
        *new_mint.key,
        *new_mint_authority.key,
        *user_wallet.key,
        *token_account_owner.key,
        *token_account.key,
        *user_wallet.key,
        *master_metadata.key,
        *metadata_mint,
        edition,
    );

    invoke_signed(
        &tx,
        &[
            new_metadata.clone(),
            new_edition.clone(),
            master_edition.clone(),
            new_mint.clone(),
            edition_marker.clone(),
            new_mint_authority.clone(),
            user_wallet.clone(),
            token_account_owner.clone(),
            token_account.clone(),
            user_wallet.clone(),
            master_metadata.clone(),
            token_program.clone(),
            system_program.clone(),
            rent.clone(),
        ],
        &[&signers_seeds],
    )?;

    Ok(())
}

/// Add zeroes to the end of the String.
/// This allows to have the size of allocated for this string memory fixed.
pub fn puffed_out_string(s: String, size: usize) -> String {
    s.to_string() + std::str::from_utf8(&vec![0u8; size - s.len()]).unwrap()
}

/// Two keys equivalence check
pub fn assert_keys_equal(key1: Pubkey, key2: Pubkey) -> ProgramResult {
    if key1 != key2 {
        Err(ErrorCode::PublicKeyMismatch.into())
    } else {
        Ok(())
    }
}
