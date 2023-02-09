use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{error::MetadataError, instruction::VerifyArgs};

pub fn verify<'a>(
    _program_id: &Pubkey,
    _accounts: &'a [AccountInfo<'a>],
    _args: VerifyArgs,
) -> ProgramResult {
    Err(MetadataError::FeatureNotSupported.into())
}
