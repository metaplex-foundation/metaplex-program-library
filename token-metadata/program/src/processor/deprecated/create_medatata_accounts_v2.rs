use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{error::MetadataError, state::DataV2};

pub fn process_create_metadata_accounts_v2<'a>(
    _program_id: &'a Pubkey,
    _accounts: &'a [AccountInfo<'a>],
    _data: DataV2,
    _is_mutable: bool,
) -> ProgramResult {
    Err(MetadataError::Removed.into())
}
