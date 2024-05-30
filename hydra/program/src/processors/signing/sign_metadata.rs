use crate::{error::HydraError, state::Fanout, utils::validation::assert_owned_by};
use anchor_lang::prelude::*;

use mpl_token_metadata::programs::MPL_TOKEN_METADATA_ID;

#[derive(Accounts)]
pub struct SignMetadata<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
    seeds = [b"fanout-config", fanout.name.as_bytes()],
    has_one = authority,
    bump
    )]
    pub fanout: Account<'info, Fanout>,
    #[account(
    constraint = fanout.account_key == holding_account.key(),
    seeds = [b"fanout-native-account", fanout.key().as_ref()],
    bump
    )]
    /// CHECK: Checked in Program
    pub holding_account: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Checked in Program
    pub metadata: UncheckedAccount<'info>,
    #[account(address=MPL_TOKEN_METADATA_ID)]
    /// CHECK: Checked in Program
    pub token_metadata_program: UncheckedAccount<'info>,
}

pub fn sign_metadata(ctx: Context<SignMetadata>) -> Result<()> {
    let metadata = ctx.accounts.metadata.to_account_info();
    let holding_account = &ctx.accounts.holding_account;
    assert_owned_by(&metadata, &MPL_TOKEN_METADATA_ID)?;
    let meta_data = metadata.try_borrow_data()?;

    if mpl_token_metadata::accounts::Metadata::from_bytes(&meta_data).is_ok() {
        return Err(HydraError::InvalidMetadata.into());
    }

    drop(meta_data);

    mpl_token_metadata::instructions::VerifyCreatorV1CpiBuilder::new(
        &ctx.accounts.token_metadata_program,
    )
    .metadata(&ctx.accounts.metadata)
    .authority(&holding_account)
    .invoke_signed(&[&[
        "fanout-native-account".as_bytes(),
        ctx.accounts.fanout.key().as_ref(),
        &[ctx.bumps.holding_account],
    ]])
    .map_err(|e| {
        error::Error::ProgramError(Box::new(ProgramErrorWithOrigin {
            program_error: e,
            error_origin: None,
            compared_values: None,
        }))
    })?;

    Ok(())
}
