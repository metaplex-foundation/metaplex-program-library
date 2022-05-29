use anchor_lang::prelude::*;
use mpl_token_metadata::utils::create_or_allocate_account_raw;

use crate::{
    constants::{FREEZE, FREEZE_FEATURE_INDEX, FREEZE_PDA_SIZE},
    set_feature_flag, CandyMachine, FreezePDA,
};

/// Set the collection PDA for the candy machine
#[derive(Accounts)]
pub struct SetFreeze<'info> {
    #[account(mut, has_one = authority)]
    candy_machine: Account<'info, CandyMachine>,
    #[account(mut)]
    authority: Signer<'info>,
    /// CHECK: account constraints checked in account trait
    #[account(mut, seeds = [FREEZE.as_bytes(), candy_machine.to_account_info().key.as_ref()], bump)]
    freeze_pda: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

pub fn handle_set_freeze(ctx: Context<SetFreeze>) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;
    if ctx.accounts.freeze_pda.data_is_empty() {
        create_or_allocate_account_raw(
            crate::id(),
            &ctx.accounts.freeze_pda.to_account_info(),
            &ctx.accounts.rent.to_account_info(),
            &ctx.accounts.system_program.to_account_info(),
            &ctx.accounts.authority.to_account_info(),
            FREEZE_PDA_SIZE,
            &[
                FREEZE.as_bytes(),
                candy_machine.key().as_ref(),
                &[*ctx.bumps.get("freeze_pda").unwrap()],
            ],
        )?;
    }
    let mut data_ref: &mut [u8] = &mut ctx.accounts.freeze_pda.try_borrow_mut_data()?;
    let mut freeze_pda_object: FreezePDA = AnchorDeserialize::deserialize(&mut &*data_ref)?;
    freeze_pda_object.candy_machine = candy_machine.key();
    freeze_pda_object.allow_thaw = false;
    freeze_pda_object.try_serialize(&mut data_ref)?;
    set_feature_flag(&mut candy_machine.data.uuid, FREEZE_FEATURE_INDEX);
    Ok(())
}
