use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::instruction::UpdateArgs;

pub fn update<'a>(
    _program_id: &Pubkey,
    _accounts: &'a [AccountInfo<'a>],
    _args: UpdateArgs,
) -> ProgramResult {
    Ok(())
}
