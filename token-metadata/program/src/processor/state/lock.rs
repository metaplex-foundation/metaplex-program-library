use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use super::toggle_asset_state;
use crate::{
    instruction::{Lock, LockArgs},
    state::TokenState,
};

pub fn lock<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: LockArgs,
) -> ProgramResult {
    let context = Lock::to_context(accounts)?;

    match args {
        LockArgs::V1 { .. } => toggle_asset_state(
            program_id,
            super::ToggleAccounts {
                payer_info: context.accounts.payer_info,
                authority_info: context.accounts.authority_info,
                token_owner_info: context.accounts.token_owner_info,
                mint_info: context.accounts.mint_info,
                token_info: context.accounts.token_info,
                master_edition_info: context.accounts.edition_info,
                metadata_info: context.accounts.metadata_info,
                token_record_info: context.accounts.token_record_info,
                system_program_info: context.accounts.system_program_info,
                sysvar_instructions_info: context.accounts.sysvar_instructions_info,
                spl_token_program_info: context.accounts.spl_token_program_info,
            },
            TokenState::Unlocked,
            TokenState::Locked,
        ),
    }
}
