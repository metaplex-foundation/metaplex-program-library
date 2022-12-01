use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::instruction::UseAssetArgs;

pub fn use_asset<'a>(
    _program_id: &Pubkey,
    _accounts: &'a [AccountInfo<'a>],
    _args: UseAssetArgs,
) -> ProgramResult {
    Ok(())
}
