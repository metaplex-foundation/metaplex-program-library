use crate::{
    error::OrArithError,
    state::{Fanout, FanoutMembershipVoucher},
};

use crate::utils::validation::*;
use anchor_lang::{
    prelude::*,
    solana_program::{sysvar, sysvar::instructions::get_instruction_relative},
};
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct UnStakeTokenMember<'info> {
    #[account(mut)]
    pub member: Signer<'info>,
    #[account(
    mut,
    seeds = [b"fanout-config", fanout.name.as_bytes()],
    bump = fanout.bump_seed,
    )]
    pub fanout: Account<'info, Fanout>,
    #[account(
    mut,
    close = member,
    seeds = [b"fanout-membership", fanout.key().as_ref(), member.key().as_ref()],
    bump,
    constraint=membership_voucher.membership_key == member.key()
    )]
    pub membership_voucher: Account<'info, FanoutMembershipVoucher>,
    #[account(
    mut,
    constraint = fanout.membership_mint.is_some() && membership_mint.key() == fanout.membership_mint.unwrap(),
    )]
    pub membership_mint: Account<'info, Mint>,
    #[account(
    mut,
    constraint = membership_mint_token_account.mint == membership_mint.key(),
    constraint = membership_mint_token_account.delegate.is_none(),
    constraint = membership_mint_token_account.close_authority.is_none(),
    )]
    pub membership_mint_token_account: Account<'info, TokenAccount>,
    #[account(
    mut,
    constraint = member_stake_account.owner == membership_voucher.key(),
    constraint = member_stake_account.mint == membership_mint.key(),
    )]
    pub member_stake_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    #[account(address = sysvar::instructions::id())]
    /// CHECK: Instructions SYSVAR
    pub instructions: UncheckedAccount<'info>,
}

pub fn unstake(ctx: Context<UnStakeTokenMember>) -> Result<()> {
    let fanout = &mut ctx.accounts.fanout;
    let member = &ctx.accounts.member;
    let ixs = &ctx.accounts.instructions;
    let membership_mint = &mut ctx.accounts.membership_mint;
    let prev_ix = get_instruction_relative(-1, ixs).unwrap();
    assert_distributed(prev_ix, member.key, fanout.membership_model)?;
    assert_owned_by(&fanout.to_account_info(), &crate::ID)?;
    assert_owned_by(&member.to_account_info(), &System::id())?;
    let amount = ctx.accounts.member_stake_account.amount;
    fanout.total_staked_shares = fanout
        .total_staked_shares
        .to_owned()
        .map(|tss| tss.checked_sub(amount).or_arith_error().unwrap());
    fanout.total_shares = membership_mint.supply;
    fanout.total_members = fanout.total_members.checked_sub(1).or_arith_error()?;
    let stake_account_info = ctx.accounts.member_stake_account.to_account_info();
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let accounts = anchor_spl::token::Transfer {
        from: stake_account_info.clone(),
        to: ctx.accounts.membership_mint_token_account.to_account_info(),
        authority: ctx.accounts.membership_voucher.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(cpi_program, accounts);
    anchor_spl::token::transfer(
        cpi_ctx.with_signer(&[&[
            "fanout-membership".as_bytes(),
            fanout.key().as_ref(),
            member.key().as_ref(),
            &[*ctx.bumps.get("membership_voucher").unwrap()],
        ]]),
        amount,
    )?;
    Ok(())
}
