use anchor_lang::prelude::*;

use crate::{
    assert_is_ata,
    constants::{FREEZE_FEATURE_INDEX, FREEZE_LOCK_FEATURE_INDEX, MAX_FREEZE_TIME},
    set_feature_flag, CandyError, CandyMachine, FreezePDA,
};

/// Set the Freeze PDA for the candy machine
#[derive(Accounts)]
pub struct SetFreeze<'info> {
    #[account(mut, has_one = authority)]
    candy_machine: Account<'info, CandyMachine>,
    #[account(mut)]
    authority: Signer<'info>,
    /// CHECK: account constraints checked in account trait
    #[account(init, seeds = [FreezePDA::PREFIX.as_bytes(), candy_machine.to_account_info().key.as_ref()], bump, space = FreezePDA::SIZE, payer = authority)]
    freeze_pda: Account<'info, FreezePDA>,
    system_program: Program<'info, System>,
    // > Only needed if spl token mint is enabled
    // freeze_ata
}

pub fn handle_set_freeze(ctx: Context<SetFreeze>, freeze_time: i64) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;
    candy_machine.assert_not_minted(error!(CandyError::NoChangingFreezeDuringMint))?;
    let freeze_pda = &mut ctx.accounts.freeze_pda;
    if freeze_time > MAX_FREEZE_TIME {
        return err!(CandyError::EnteredFreezeIsMoreThanMaxFreeze);
    }
    freeze_pda.init(candy_machine.key(), None, freeze_time);

    if let Some(mint_pubkey) = candy_machine.token_mint {
        let freeze_ata = ctx
            .remaining_accounts
            .get(0)
            .ok_or(CandyError::MissingFreezeAta)?;
        assert_is_ata(freeze_ata, freeze_pda.to_account_info().key, &mint_pubkey)
            .map_err(|_| CandyError::IncorrectFreezeAta)?;
    }
    set_feature_flag(&mut candy_machine.data.uuid, FREEZE_FEATURE_INDEX);
    set_feature_flag(&mut candy_machine.data.uuid, FREEZE_LOCK_FEATURE_INDEX);
    Ok(())
}
