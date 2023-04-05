use crate::{
    error::HydraError,
    state::{Fanout, FanoutMembershipVoucher, MembershipModel},
};

use crate::utils::logic::{
    calculation::calculate_payer_rewards,
    transfer::{transfer_from_mint_holding, transfer_native},
};

use crate::utils::logic::distribution::{distribute_mint, distribute_native};

use crate::{state::FanoutMembershipMintVoucher, utils::validation::*};
use clockwork_sdk::state::{Thread, ThreadAccount, ThreadResponse};

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
#[instruction(distribute_for_mint: bool)]
pub struct DistributeClockNftMember<'info> {
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
    /// CHECK: Checked in program
    pub member: UncheckedAccount<'info>,
    #[
    account(
    mut,
    address = fanout.authority
    )]
    /// CHECK: Restricted to fanout
    pub authority: UncheckedAccount<'info>,
    #[
    account(
    mut,
    constraint = membership_mint_token_account.delegate.is_none(),
    constraint = membership_mint_token_account.close_authority.is_none(),
    constraint = membership_mint_token_account.mint == membership_key.key(),
    )]
    pub membership_mint_token_account: Box<Account<'info, TokenAccount>>,
    pub membership_key: Box<Account<'info, Mint>>,
    #[account(
    mut,
    seeds = [b"fanout-membership", fanout.key().as_ref(), membership_key.key().as_ref()],
    constraint = membership_voucher.membership_key == membership_key.key(),
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
    /// CHECK: Could be a native or Token Account
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
    #[account(mut)]
    pub payer_token_account: Box<Account<'info, TokenAccount>>,
}

pub fn distribute_clock_for_nft(
    ctx: Context<DistributeClockNftMember>,
    distribute_for_mint: bool,
) -> Result<ThreadResponse> {
    let fanout = &mut ctx.accounts.fanout;
    let fanout_info = fanout.to_account_info();
    let membership_voucher = &mut ctx.accounts.membership_voucher;
    let membership_voucher_info = membership_voucher.to_account_info();
    let member = &mut ctx.accounts.member;
    let authority = &mut ctx.accounts.authority;
    let membership_mint_token_account = &ctx.accounts.membership_mint_token_account;
    let membership_key = &ctx.accounts.membership_key;
    let payer_token_account = &mut ctx.accounts.payer_token_account;
    let payer = &mut ctx.accounts.payer;
    assert_owned_by(&fanout_info, &crate::ID)?;
    assert_owned_by(&membership_voucher_info, &crate::ID)?;
    assert_owned_by(&member.to_account_info(), &System::id())?;
    assert_owned_by(&authority.to_account_info(), &System::id())?;
    assert_owned_by(&payer.to_account_info(), &System::id())?;
    assert_owned_by(
        &payer_token_account.to_account_info(),
        &ctx.accounts.token_program.key(),
    )?;
    assert_membership_model(fanout, MembershipModel::NFT)?;
    assert_shares_distributed(fanout)?;
    assert_holding(
        &member.to_account_info(),
        membership_mint_token_account,
        &membership_key.to_account_info(),
    )?;
    let rewards: u64 = fanout.payer_reward_basis_points | 666;

    let payer_rewards = calculate_payer_rewards(fanout.total_inflow, rewards)?;

    if distribute_for_mint {
        if payer_rewards > 0 as u64 {
            transfer_from_mint_holding(
                &ctx.accounts.fanout,
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
            &mut ctx.accounts.fanout,
            &mut ctx.accounts.membership_voucher,
            ctx.accounts.rent.to_owned(),
            ctx.accounts.system_program.to_owned(),
            ctx.accounts.token_program.to_owned(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.member.to_owned(),
            &ctx.accounts.membership_key.key(),
        )?;
    } else {
        if payer_rewards > 0 {
            let current_snapshot = &mut ctx.accounts.holding_account.lamports();
            transfer_native(
                ctx.accounts.holding_account.to_account_info(),
                payer.to_account_info(),
                *current_snapshot,
                payer_rewards,
            );
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
pub struct DistributeNftMember<'info> {
    pub payer: Signer<'info>,
    #[account(mut)]
    /// CHECK: Checked in program
    pub member: UncheckedAccount<'info>,
    #[
    account(
    mut,
    constraint = membership_mint_token_account.delegate.is_none(),
    constraint = membership_mint_token_account.close_authority.is_none(),
    constraint = membership_mint_token_account.mint == membership_key.key(),
    )]
    pub membership_mint_token_account: Account<'info, TokenAccount>,
    pub membership_key: Account<'info, Mint>,
    #[account(
    mut,
    seeds = [b"fanout-membership", fanout.key().as_ref(), membership_key.key().as_ref()],
    constraint = membership_voucher.membership_key == membership_key.key(),
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
    /// CHECK: Could be a native or Token Account
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
}

pub fn distribute_for_nft(
    ctx: Context<DistributeNftMember>,
    distribute_for_mint: bool,
) -> Result<()> {
    let fanout = &mut ctx.accounts.fanout;
    let fanout_info = fanout.to_account_info();
    let membership_voucher = &mut ctx.accounts.membership_voucher;
    let membership_voucher_info = membership_voucher.to_account_info();
    let member = &mut ctx.accounts.member;
    let membership_mint_token_account = &ctx.accounts.membership_mint_token_account;
    let membership_key = &ctx.accounts.membership_key;
    assert_owned_by(&fanout_info, &crate::ID)?;
    assert_owned_by(&membership_voucher_info, &crate::ID)?;
    assert_owned_by(&member.to_account_info(), &System::id())?;
    assert_membership_model(fanout, MembershipModel::NFT)?;
    assert_shares_distributed(fanout)?;
    assert_holding(
        &member.to_account_info(),
        membership_mint_token_account,
        &membership_key.to_account_info(),
    )?;
    if distribute_for_mint {
        distribute_mint(
            *ctx.accounts.fanout_mint.to_owned(),
            &mut ctx.accounts.fanout_for_mint,
            &mut ctx.accounts.fanout_for_mint_membership_voucher,
            &mut ctx.accounts.fanout_mint_member_token_account,
            &mut ctx.accounts.holding_account,
            &mut ctx.accounts.fanout,
            &mut ctx.accounts.membership_voucher,
            ctx.accounts.rent.to_owned(),
            ctx.accounts.system_program.to_owned(),
            ctx.accounts.token_program.to_owned(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.member.to_owned(),
            &ctx.accounts.membership_key.key(),
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
