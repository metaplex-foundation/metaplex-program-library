use arch_lang::prelude::*;

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
    /// Remaining accounts (e.g., token mint)
    #[account(optional)]
    token_mint: Option<Account<'info, Token>>,
}

#[account]
pub struct CandyMachine {
    authority: Pubkey,
    wallet: Pubkey,
    token_mint: Option<Pubkey>,
    data: CandyMachineData,
}

#[derive(Clone, Default, AnchorSerialize, AnchorDeserialize)]
pub struct CandyMachineData {
    pub items_available: u64,
    pub uuid: String,
    pub hidden_settings: Option<bool>,
    pub retain_authority: bool,
}

/// Errors for Candy Machine
#[error_code]
pub enum CandyError {
    NoChangingAuthorityWithCollection,
    CannotChangeNumberOfLines,
    CannotSwitchToHiddenSettings,
    CannotSwitchFromHiddenSettings,
    NoChangingTokenWithFreeze,
    CandyCollectionRequiresRetainAuthority,
}

pub fn handle_update_authority(
    ctx: Context<UpdateCandyMachine>,
    new_authority: Option<Pubkey>,
) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;

    // Do not allow changing update authority if collections feature is active
    if let Some(new_auth) = new_authority {
        if is_feature_active(&candy_machine.data.uuid, COLLECTIONS_FEATURE_INDEX) {
            return Err(error!(CandyError::NoChangingAuthorityWithCollection));
        } else {
            candy_machine.authority = new_auth;
        }
    }
    Ok(())
}

pub fn handle_update_candy_machine(
    ctx: Context<UpdateCandyMachine>,
    data: CandyMachineData,
) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;

    // Prevent changing number of items if hidden settings are not enabled
    if data.items_available != candy_machine.data.items_available && data.hidden_settings.is_none() {
        return Err(error!(CandyError::CannotChangeNumberOfLines));
    }

    let token_mint = ctx.accounts.token_mint.as_ref().map(|account| account.key());

    if candy_machine.data.items_available > 0
        && candy_machine.data.hidden_settings.is_none()
        && data.hidden_settings.is_some()
    {
        return Err(error!(CandyError::CannotSwitchToHiddenSettings));
    }

    if candy_machine.data.hidden_settings.is_some() && data.hidden_settings.is_none() {
        return Err(error!(CandyError::CannotSwitchFromHiddenSettings));
    }

    let old_uuid = candy_machine.data.uuid.clone();
    if is_feature_active(&old_uuid, FREEZE_FEATURE_INDEX) && candy_machine.token_mint != token_mint {
        return Err(error!(CandyError::NoChangingTokenWithFreeze));
    }
    if is_feature_active(&old_uuid, COLLECTIONS_FEATURE_INDEX) && !data.retain_authority {
        return Err(error!(CandyError::CandyCollectionRequiresRetainAuthority));
    }

    // Update state
    candy_machine.wallet = ctx.accounts.wallet.key();
    candy_machine.data = data;
    candy_machine.data.uuid = old_uuid;
    candy_machine.token_mint = token_mint;

    Ok(())
}

/// Dummy function to check if a feature is active
fn is_feature_active(uuid: &str, feature_index: u8) -> bool {
    // Implement your logic to determine if a feature is active
    false
}
