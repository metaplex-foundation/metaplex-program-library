use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::instruction::MigrateArgs;

pub fn migrate<'a>(
    _program_id: &Pubkey,
    _accounts: &'a [AccountInfo<'a>],
    _args: MigrateArgs,
) -> ProgramResult {
    Ok(())
}
