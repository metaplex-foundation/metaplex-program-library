use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{error::MetadataError, state::Data};

pub fn process_deprecated_create_metadata_accounts<'a>(
    _program_id: &'a Pubkey,
    _accounts: &'a [AccountInfo<'a>],
    _data: Data,
    _is_mutable: bool,
) -> ProgramResult {
    Err(MetadataError::Removed.into())
}
