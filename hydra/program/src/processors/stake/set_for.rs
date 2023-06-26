use crate::error::{HydraError, OrArithError};
use crate::state::{Fanout, FanoutMembershipVoucher, FANOUT_MEMBERSHIP_VOUCHER_SIZE};

use crate::utils::validation::*;
use crate::MembershipModel;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
#[derive(Accounts)]
#[instruction(shares: u64)]
pub struct SetForTokenMemberStake<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: Native Account
    pub member: UncheckedAccount<'info>,
    #[account(
    mut,
    seeds = [b"fanout-config", fanout.name.as_bytes()],
    bump = fanout.bump_seed,
    )]
    pub fanout: Account<'info, Fanout>,
    #[account(
    init,
    space = FANOUT_MEMBERSHIP_VOUCHER_SIZE,
    seeds = [b"fanout-membership", fanout.key().as_ref(), member.key().as_ref()],
    bump,
    payer = authority
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
    constraint = membership_mint_token_account.amount >= shares,
    constraint = membership_mint_token_account.owner == authority.key()
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
}

pub fn set_for_token_member_stake(ctx: Context<SetForTokenMemberStake>, shares: u64) -> Result<()> {
    let fanout = &mut ctx.accounts.fanout;
    let member = &ctx.accounts.member;
    let membership_voucher = &mut ctx.accounts.membership_voucher;
    let membership_mint = &mut ctx.accounts.membership_mint;
    assert_owned_by(&fanout.to_account_info(), &crate::ID)?;
    assert_owned_by(&member.to_account_info(), &System::id())?;
    assert_membership_model(fanout, MembershipModel::Token)?;
    assert_ata(
        &ctx.accounts.member_stake_account.to_account_info(),
        &membership_voucher.key(),
        &membership_mint.key(),
        Some(HydraError::InvalidStakeAta.into()),
    )?;
    membership_voucher.stake_time = Clock::get()?.unix_timestamp;
    membership_voucher.fanout = fanout.key();
    membership_voucher.membership_key = member.key();
    fanout.total_staked_shares = fanout
        .total_staked_shares
        .and_then(|ss| ss.checked_add(shares));
    fanout.total_shares = membership_mint.supply;
    fanout.total_members = fanout.total_members.checked_add(1).or_arith_error()?;
    membership_voucher.shares = shares;
    membership_voucher.bump_seed = *ctx.bumps.get("membership_voucher").unwrap();
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let accounts = anchor_spl::token::Transfer {
        from: ctx.accounts.membership_mint_token_account.to_account_info(),
        to: ctx.accounts.member_stake_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, accounts);
    anchor_spl::token::transfer(cpi_ctx, shares)?;
    Ok(())
}
