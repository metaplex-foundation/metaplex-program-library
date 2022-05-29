use anchor_lang::prelude::*;

use crate::{
    constants::{FREEZE, FREEZE_FEATURE_INDEX},
    remove_feature_flag, CandyMachine, FreezePDA,
};

/// removes the freeze flag from candy machine without closing the freeze pda
#[derive(Accounts)]
pub struct RemoveFreeze<'info> {
    #[account(mut, has_one = authority)]
    candy_machine: Account<'info, CandyMachine>,
    authority: Signer<'info>,
    #[account(seeds = [FREEZE.as_bytes(), candy_machine.to_account_info().key.as_ref()], bump)]
    freeze_pda: Account<'info, FreezePDA>,
}

pub fn handle_remove_freeze(ctx: Context<RemoveFreeze>) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;
    let freeze_pda = &mut ctx.accounts.freeze_pda;
    freeze_pda.allow_thaw = true;
    remove_feature_flag(&mut candy_machine.data.uuid, FREEZE_FEATURE_INDEX);
    Ok(())
}
