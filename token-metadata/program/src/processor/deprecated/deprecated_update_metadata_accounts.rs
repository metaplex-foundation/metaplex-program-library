
use solana_program::{
    account_info::{AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::{
    error::MetadataError,
    state::{Data},
};

/// Update existing account instruction
pub fn process_deprecated_update_metadata_accounts(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _optional_data: Option<Data>,
    _update_authority: Option<Pubkey>,
    _primary_sale_happened: Option<bool>,
) -> ProgramResult {
    Err(MetadataError::Removed.into())
}
