use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::error::MetadataError;

pub fn process_deprecated_mint_new_edition_from_master_edition_via_vault_proxy<'a>(
    _program_id: &'a Pubkey,
    _accounts: &'a [AccountInfo<'a>],
    _edition: u64,
) -> ProgramResult {
    Err(MetadataError::Removed.into())
}
