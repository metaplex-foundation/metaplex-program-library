use anchor_lang::{prelude::*, AccountsClose};

use crate::{
    constants::{FREEZE_FEATURE_INDEX, FREEZE_LOCK_FEATURE_INDEX},
    remove_feature_flag, CandyError, CandyMachine, FreezePDA,
};

/// Removes the freeze flag from candy machine without closing the freeze pda unless no NFTs have been minted
#[derive(Accounts)]
pub struct RemoveFreeze<'info> {
    #[account(mut, has_one = authority)]
    candy_machine: Account<'info, CandyMachine>,
    #[account(mut)]
    authority: Signer<'info>,
    #[account(mut, seeds = [FreezePDA::PREFIX.as_bytes(), candy_machine.to_account_info().key.as_ref()], bump)]
    freeze_pda: Account<'info, FreezePDA>,
}

pub fn handle_remove_freeze(ctx: Context<RemoveFreeze>) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;
    let freeze_pda = &mut ctx.accounts.freeze_pda;
    freeze_pda.allow_thaw = true;
    remove_feature_flag(&mut candy_machine.data.uuid, FREEZE_FEATURE_INDEX);

    // Closes the account to enable editing if minting hasn't started
    if candy_machine
        .assert_not_minted(error!(CandyError::Uninitialized))
        .is_ok()
    {
        freeze_pda.close(ctx.accounts.authority.to_account_info())?;
        remove_feature_flag(&mut candy_machine.data.uuid, FREEZE_LOCK_FEATURE_INDEX);
    }
    Ok(())
}
