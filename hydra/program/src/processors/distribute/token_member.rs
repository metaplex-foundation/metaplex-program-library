use crate::error::HydraError;

use crate::state::{Fanout, FanoutMembershipVoucher, MembershipModel};

use crate::{
    utils::logic::{
        calculation::calculate_payer_rewards,
        distribution::{distribute_mint, distribute_native},
        transfer::{transfer_from_mint_holding, transfer_native},
    },
};

use crate::utils::validation::*;

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use clockwork_sdk::state::{Thread, ThreadAccount, ThreadResponse};

#[derive(Accounts)]
#[instruction(distribute_for_mint: bool)]
pub struct DistributeClockTokenMember<'info> {
    #[account(
        signer,
        address = hydra.pubkey(),
        constraint = hydra.authority == payer.key()
    )]
    pub hydra: Box<Account<'info, Thread>>,
    #[account(mut)]
    /// CHECK: Checked in Program
    pub payer: UncheckedAccount<'info>,
    #[account(mut)]

    /// CHECK: Optional Account
    pub member: UncheckedAccount<'info>,
    #[
    account(
    mut,
    address = fanout.authority
    )]
    /// CHECK: Restricted to fanout
    pub authority: UncheckedAccount<'info>,
    #[account(
    mut,
    seeds = [b"fanout-membership", fanout.key().as_ref(), member.key().as_ref()],
    constraint = membership_voucher.membership_key == member.key(),
    bump = membership_voucher.bump_seed,
    )]
    pub membership_voucher: Account<'info, FanoutMembershipVoucher>,
    #[account(
    mut,
    seeds = [b"fanout-config", fanout.name.as_bytes()],
    bump = fanout.bump_seed,
    )]
    pub fanout: Box<Account<'info, Fanout>>,
    #[account(mut)]
    /// CHECK: Could be native or Token Account
    pub holding_account: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Optional Account
    pub fanout_for_mint: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Optional Account
    pub fanout_for_mint_membership_voucher: UncheckedAccount<'info>,
    pub fanout_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    /// CHECK: Optional Account
    pub fanout_mint_member_token_account: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    #[account(
    mut,
    constraint = fanout.membership_mint.is_some() && membership_mint.key() == fanout.membership_mint.unwrap(),
    )]
    pub membership_mint: Box<Account<'info, Mint>>,
    #[account(
    mut,
    constraint = member_stake_account.owner == membership_voucher.key(),
    constraint = member_stake_account.mint == membership_mint.key(),
    constraint = member_stake_account.amount > 0
    )]
    pub member_stake_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub payer_token_account: Box<Account<'info, TokenAccount>>,
}

pub fn distribute_clock_for_token(
    ctx: Context<DistributeClockTokenMember>,
    distribute_for_mint: bool,
) -> Result<ThreadResponse> {
    let fanout = &mut ctx.accounts.fanout;
    let fanout_info = fanout.to_account_info();
    let membership_voucher = &mut ctx.accounts.membership_voucher;
    let membership_voucher_info = membership_voucher.to_account_info();
    let member = &mut ctx.accounts.member;
    let authority = &mut ctx.accounts.authority;
    let membership_mint = &ctx.accounts.membership_mint;
    let payer_token_account = &mut ctx.accounts.payer_token_account;
    let payer = &mut ctx.accounts.payer;
    fanout.total_shares = membership_mint.supply;
    assert_ata(
        &ctx.accounts.member_stake_account.to_account_info(),
        &membership_voucher.key(),
        &membership_mint.key(),
        Some(HydraError::InvalidStakeAta.into()),
    )?;
    assert_owned_by(&fanout_info, &crate::ID)?;
    assert_owned_by(&membership_voucher_info, &crate::ID)?;
    assert_owned_by(&member.to_account_info(), &System::id())?;
    assert_owned_by(&authority.to_account_info(), &System::id())?;
    assert_owned_by(&payer.to_account_info(), &System::id())?;
    assert_owned_by(
        &payer_token_account.to_account_info(),
        &ctx.accounts.token_program.key(),
    )?;
    assert_membership_model(fanout, MembershipModel::Token)?;
    assert_shares_distributed(fanout)?;
    let rewards: u64 = fanout.payer_reward_basis_points | 666;

    let payer_rewards = calculate_payer_rewards(fanout.total_inflow, rewards)?;

    if distribute_for_mint {
        if payer_rewards > 0 as u64 {
            transfer_from_mint_holding(
                &fanout,
                authority.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.holding_account.to_account_info(),
                payer_token_account.to_account_info(),
                payer_rewards,
            )?;
        }
        distribute_mint(
            *ctx.accounts.fanout_mint.to_owned(),
            &mut ctx.accounts.fanout_for_mint,
            &mut ctx.accounts.fanout_for_mint_membership_voucher,
            &mut ctx.accounts.fanout_mint_member_token_account,
            &mut ctx.accounts.holding_account,
            fanout,
            &mut ctx.accounts.membership_voucher,
            ctx.accounts.rent.to_owned(),
            ctx.accounts.system_program.to_owned(),
            ctx.accounts.token_program.to_owned(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.member.to_owned(),
            &ctx.accounts.member.key(),
        )?;
    } else {
        if payer_rewards > 0 {
            let current_snapshot = &mut ctx.accounts.holding_account.lamports();
            transfer_native(
                ctx.accounts.holding_account.to_account_info(),
                payer.to_account_info(),
                *current_snapshot,
                payer_rewards,
            )?;
        }
        distribute_native(
            &mut ctx.accounts.holding_account,
            &mut ctx.accounts.fanout,
            &mut ctx.accounts.membership_voucher,
            ctx.accounts.member.to_owned(),
            ctx.accounts.rent.to_owned(),
        )?;
    }
    Ok(ThreadResponse {
        next_instruction: None,
        kickoff_instruction: None,
    })
}

#[derive(Accounts)]
#[instruction(distribute_for_mint: bool)]
pub struct DistributeTokenMember<'info> {
    pub payer: Signer<'info>,
    #[account(mut)]
    /// CHECK: Optional Account
    pub member: UncheckedAccount<'info>,
    #[account(
    mut,
    seeds = [b"fanout-membership", fanout.key().as_ref(), member.key().as_ref()],
    constraint = membership_voucher.membership_key == member.key(),
    bump = membership_voucher.bump_seed,
    )]
    pub membership_voucher: Account<'info, FanoutMembershipVoucher>,
    #[account(
    mut,
    seeds = [b"fanout-config", fanout.name.as_bytes()],
    bump = fanout.bump_seed,
    )]
    pub fanout: Box<Account<'info, Fanout>>,
    #[account(mut)]
    /// CHECK: Could be native or Token Account
    pub holding_account: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Optional Account
    pub fanout_for_mint: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Optional Account
    pub fanout_for_mint_membership_voucher: UncheckedAccount<'info>,
    pub fanout_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    /// CHECK: Optional Account
    pub fanout_mint_member_token_account: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    #[account(
    mut,
    constraint = fanout.membership_mint.is_some() && membership_mint.key() == fanout.membership_mint.unwrap(),
    )]
    pub membership_mint: Box<Account<'info, Mint>>,
    #[account(
    mut,
    constraint = member_stake_account.owner == membership_voucher.key(),
    constraint = member_stake_account.mint == membership_mint.key(),
    constraint = member_stake_account.amount > 0
    )]
    pub member_stake_account: Box<Account<'info, TokenAccount>>,
}
pub fn distribute_for_token(
    ctx: Context<DistributeTokenMember>,
    distribute_for_mint: bool,
) -> Result<()> {
    let fanout = &mut ctx.accounts.fanout;
    let fanout_info = fanout.to_account_info();
    let membership_voucher = &mut ctx.accounts.membership_voucher;
    let membership_voucher_info = membership_voucher.to_account_info();
    let member = &mut ctx.accounts.member;
    let membership_mint = &ctx.accounts.membership_mint;
    fanout.total_shares = membership_mint.supply;
    assert_ata(
        &ctx.accounts.member_stake_account.to_account_info(),
        &membership_voucher.key(),
        &membership_mint.key(),
        Some(HydraError::InvalidStakeAta.into()),
    )?;
    assert_owned_by(&fanout_info, &crate::ID)?;
    assert_owned_by(&membership_voucher_info, &crate::ID)?;
    assert_owned_by(&member.to_account_info(), &System::id())?;
    assert_membership_model(fanout, MembershipModel::Token)?;
    assert_shares_distributed(fanout)?;

    if distribute_for_mint {
        distribute_mint(
            *ctx.accounts.fanout_mint.to_owned(),
            &mut ctx.accounts.fanout_for_mint,
            &mut ctx.accounts.fanout_for_mint_membership_voucher,
            &mut ctx.accounts.fanout_mint_member_token_account,
            &mut ctx.accounts.holding_account,
            fanout,
            &mut ctx.accounts.membership_voucher,
            ctx.accounts.rent.to_owned(),
            ctx.accounts.system_program.to_owned(),
            ctx.accounts.token_program.to_owned(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.member.to_owned(),
            &ctx.accounts.member.key(),
        )?;
    } else {
        distribute_native(
            &mut ctx.accounts.holding_account,
            &mut ctx.accounts.fanout,
            &mut ctx.accounts.membership_voucher,
            ctx.accounts.member.to_owned(),
            ctx.accounts.rent.to_owned(),
        )?;
    }
    Ok(())
}
