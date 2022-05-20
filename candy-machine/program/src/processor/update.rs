use anchor_lang::prelude::*;

use crate::{
    constants::COLLECTIONS_FEATURE_INDEX, is_feature_active, CandyError, CandyMachine,
    CandyMachineData,
};

/// Update the candy machine state.
#[derive(Accounts)]
pub struct UpdateCandyMachine<'info> {
    #[account(
    mut,
    has_one = authority
    )]
    candy_machine: Account<'info, CandyMachine>,
    authority: Signer<'info>,
    /// CHECK: wallet can be any account and is not written to or read
    wallet: UncheckedAccount<'info>,
}

pub fn handle_update_authority(
    ctx: Context<UpdateCandyMachine>,
    new_authority: Option<Pubkey>,
) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;

    if let Some(new_auth) = new_authority {
        candy_machine.authority = new_auth;
    }

    Ok(())
}

// updates without modifying UUID
pub fn handle_update_candy_machine(
    ctx: Context<UpdateCandyMachine>,
    data: CandyMachineData,
) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;

    if data.items_available != candy_machine.data.items_available && data.hidden_settings.is_none()
    {
        return err!(CandyError::CannotChangeNumberOfLines);
    }

    if candy_machine.data.items_available > 0
        && candy_machine.data.hidden_settings.is_none()
        && data.hidden_settings.is_some()
    {
        return err!(CandyError::CannotSwitchToHiddenSettings);
    }

    let old_uuid = candy_machine.data.uuid.clone();
    candy_machine.wallet = ctx.accounts.wallet.key();
    if is_feature_active(&old_uuid, COLLECTIONS_FEATURE_INDEX) && !data.retain_authority {
        return err!(CandyError::CandyCollectionRequiresRetainAuthority);
    }
    candy_machine.data = data;
    candy_machine.data.uuid = old_uuid;

    if !ctx.remaining_accounts.is_empty() {
        candy_machine.token_mint = Some(ctx.remaining_accounts[0].key())
    } else {
        candy_machine.token_mint = None;
    }
    Ok(())
}
