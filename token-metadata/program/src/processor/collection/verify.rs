use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::instruction::VerifyArgs;

pub fn verify<'a>(
    _program_id: &Pubkey,
    _accounts: &'a [AccountInfo<'a>],
    _args: VerifyArgs,
) -> ProgramResult {
    Ok(())
}
