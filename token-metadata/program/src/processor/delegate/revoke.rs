use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::instruction::RevokeArgs;

pub fn revoke<'a>(
    _program_id: &Pubkey,
    _accounts: &'a [AccountInfo<'a>],
    _args: RevokeArgs,
) -> ProgramResult {
    Ok(())
}
