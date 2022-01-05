//! Module provide runtime utilities

use crate::{id, ErrorCode};
use anchor_lang::{
    prelude::*,
    solana_program::{
        program::{invoke, invoke_signed},
        system_instruction,
    },
};
use std::convert::TryInto;

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

/// Create account almost from scratch, lifted from
/// https://github.com/solana-labs/solana-program-library/tree/master/associated-token-account/program/src/processor.rs#L51-L98
#[inline(always)]
pub fn create_or_allocate_account_raw<'a>(
    program_id: Pubkey,
    new_account_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    size: usize,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    let required_lamports = rent
        .minimum_balance(size)
        .max(1)
        .saturating_sub(new_account_info.lamports());

    if required_lamports > 0 {
        msg!("Transfer {} lamports to the new account", required_lamports);
        invoke(
            &system_instruction::transfer(&payer_info.key, new_account_info.key, required_lamports),
            &[
                payer_info.clone(),
                new_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;
    }

    let accounts = &[new_account_info.clone(), system_program_info.clone()];

    msg!("Allocate space for the account");
    invoke_signed(
        &system_instruction::allocate(new_account_info.key, size.try_into().unwrap()),
        accounts,
        &[&signer_seeds],
    )?;

    msg!("Assign the account to the owning program");
    invoke_signed(
        &system_instruction::assign(new_account_info.key, &program_id),
        accounts,
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

pub fn puffed_out_string(s: &String, size: usize) -> String {
    let mut array_of_zeroes = vec![];
    let puff_amount = size - s.len();
    while array_of_zeroes.len() < puff_amount {
        array_of_zeroes.push(0u8);
    }
    s.clone() + std::str::from_utf8(&array_of_zeroes).unwrap()
}
