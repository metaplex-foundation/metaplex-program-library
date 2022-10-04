use anchor_lang::prelude::*;

use crate::{
    constants::{COLLECTIONS_FEATURE_INDEX, FREEZE_FEATURE_INDEX},
    is_feature_active, CandyError, CandyMachine, CandyMachineData,
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
    // Remaining accounts.
    // token mint
}

pub fn handle_update_authority(
    ctx: Context<UpdateCandyMachine>,
    new_authority: Option<Pubkey>,
) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;

    // Do not allow changing update authority if collections is active
    if let Some(new_auth) = new_authority {
        if is_feature_active(&candy_machine.data.uuid, COLLECTIONS_FEATURE_INDEX) {
            return err!(CandyError::NoChangingAuthorityWithCollection);
        } else {
            candy_machine.authority = new_auth;
        }
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

    let token_mint = ctx
        .remaining_accounts
        .get(0)
        .map(|account_info| account_info.key());

    if candy_machine.data.items_available > 0
        && candy_machine.data.hidden_settings.is_none()
        && data.hidden_settings.is_some()
    {
        return err!(CandyError::CannotSwitchToHiddenSettings);
    }

    if candy_machine.data.hidden_settings.is_some() && data.hidden_settings.is_none() {
        return err!(CandyError::CannotSwitchFromHiddenSettings);
    }

    let old_uuid = candy_machine.data.uuid.clone();
    if is_feature_active(&old_uuid, FREEZE_FEATURE_INDEX) && candy_machine.token_mint != token_mint
    {
        return err!(CandyError::NoChangingTokenWithFreeze);
    }
    if is_feature_active(&old_uuid, COLLECTIONS_FEATURE_INDEX) && !data.retain_authority {
        return err!(CandyError::CandyCollectionRequiresRetainAuthority);
    }

    candy_machine.wallet = ctx.accounts.wallet.key();
    candy_machine.data = data;
    candy_machine.data.uuid = old_uuid;
    candy_machine.token_mint = token_mint;

    Ok(())
}
