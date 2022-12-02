use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::instruction::DelegateArgs;

pub fn delegate<'a>(
    _program_id: &Pubkey,
    _accounts: &'a [AccountInfo<'a>],
    _args: DelegateArgs,
) -> ProgramResult {
    Ok(())
}
