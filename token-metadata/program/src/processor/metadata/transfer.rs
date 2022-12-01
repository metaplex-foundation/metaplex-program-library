use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::instruction::TransferArgs;

pub fn transfer<'a>(
    _program_id: &Pubkey,
    _accounts: &'a [AccountInfo<'a>],
    _args: TransferArgs,
) -> ProgramResult {
    Ok(())
}
