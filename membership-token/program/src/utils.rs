//! Module provide runtime utilities

use crate::{id, ErrorCode};
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

pub const STRING_DEFAULT_SIZE: usize = 20;
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

/// Runtime check of `spl_token` `Account` owner.
pub fn assert_spl_token_account_owner(
    account: &AccountInfo,
    owner: &Pubkey,
) -> Result<(), ProgramError> {
    let account = TokenAccount::try_deserialize_unchecked(&mut account.data.borrow().as_ref())?;
    if account.owner != *owner {
        return Err(ProgramError::IllegalOwner);
    }

    Ok(())
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
