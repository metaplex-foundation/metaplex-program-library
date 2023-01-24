use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{error::MetadataError, instruction::BurnArgs};

pub fn burn<'a>(
    _program_id: &Pubkey,
    _accounts: &'a [AccountInfo<'a>],
    _args: BurnArgs,
) -> ProgramResult {
    Err(MetadataError::FeatureNotSupported.into())
}
