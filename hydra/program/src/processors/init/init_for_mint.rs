use crate::{
    error::HydraError,
    state::{Fanout, FanoutMint},
    utils::validation::assert_ata,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

#[derive(Accounts)]
#[instruction(bump_seed: u8)]
pub struct InitializeFanoutForMint<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
    mut,
    seeds = [b"fanout-config", fanout.name.as_bytes()],
    has_one = authority,
    bump = fanout.bump_seed,
    )]
    pub fanout: Account<'info, Fanout>,
    #[account(
    init,
    payer= authority,
    space = 200,
    seeds = [b"fanout-config", fanout.key().as_ref(), mint.key().as_ref()],
    bump
    )]
    pub fanout_for_mint: Account<'info, FanoutMint>,
    #[account(
    mut,
    constraint = mint_holding_account.owner == fanout.key(),
    constraint = mint_holding_account.delegate.is_none(),
    constraint = mint_holding_account.close_authority.is_none(),
    constraint = mint_holding_account.mint == mint.key(),
    )
    ]
    pub mint_holding_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn init_for_mint(ctx: Context<InitializeFanoutForMint>, bump_seed: u8) -> Result<()> {
    let fanout_mint = &mut ctx.accounts.fanout_for_mint;
    let fanout = &ctx.accounts.fanout;
    let mint_holding_account = &ctx.accounts.mint_holding_account;
    fanout_mint.fanout = fanout.to_account_info().key();
    fanout_mint.total_inflow = mint_holding_account.amount;
    fanout_mint.last_snapshot_amount = mint_holding_account.amount;
    fanout_mint.bump_seed = bump_seed;
    fanout_mint.mint = ctx.accounts.mint.to_account_info().key();
    assert_ata(
        &mint_holding_account.to_account_info(),
        &fanout.key(),
        &ctx.accounts.mint.key(),
        Some(HydraError::HoldingAccountMustBeAnATA.into()),
    )?;
    fanout_mint.token_account = mint_holding_account.to_account_info().key();
    Ok(())
}
