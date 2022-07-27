use anchor_lang::prelude::*;

use crate::{
    constants::LOCKUP_SETTINGS_FEATURE_INDEX, remove_feature_flag, state::LOCKUP_SETTINGS_SEED,
    CandyError, CandyMachine, LockupSettings,
};

/// Set the collection PDA for the candy machine
#[derive(Accounts)]
pub struct CloseLockupSettings<'info> {
    /// CHECK: account may be empty
    #[account(mut)]
    candy_machine: UncheckedAccount<'info>,
    authority: Signer<'info>,
    #[account(
        mut,
        close = authority,
        seeds = [LOCKUP_SETTINGS_SEED.as_bytes(), candy_machine.to_account_info().key.as_ref()],
        bump
    )]
    lockup_settings: Box<Account<'info, LockupSettings>>,
    system_program: Program<'info, System>,
}

pub fn handle_close_lockup_settings(ctx: Context<CloseLockupSettings>) -> Result<()> {
    let candy_machine_info = &ctx.accounts.candy_machine;
    if !candy_machine_info.data_is_empty() {
        let candy_machine = &mut Account::<CandyMachine>::try_from(candy_machine_info)?;
        if candy_machine.authority != ctx.accounts.authority.key() {
            return err!(CandyError::InvalidCandyMachineAuthority);
        }
        remove_feature_flag(&mut candy_machine.data.uuid, LOCKUP_SETTINGS_FEATURE_INDEX);
    }
    Ok(())
}
