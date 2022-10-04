use crate::error::HydraError;
use crate::state::{Fanout, FanoutMembershipVoucher};

use crate::MembershipModel;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(shares: u64)]
pub struct TransferShares<'info> {
    pub authority: Signer<'info>,
    /// CHECK: Native Account
    pub from_member: UncheckedAccount<'info>,
    /// CHECK: Native Account
    pub to_member: UncheckedAccount<'info>,
    #[account(
    mut,
    seeds = [b"fanout-config", fanout.name.as_bytes()],
    has_one = authority,
    bump = fanout.bump_seed,
    )]
    pub fanout: Account<'info, Fanout>,
    #[account(
    mut,
    seeds = [b"fanout-membership", fanout.key().as_ref(), from_member.key().as_ref()],
    bump,
    has_one = fanout,
    )]
    pub from_membership_account: Account<'info, FanoutMembershipVoucher>,
    #[account(
    mut,
    seeds = [b"fanout-membership", fanout.key().as_ref(), to_member.key().as_ref()],
    bump,
    has_one = fanout,
    )]
    pub to_membership_account: Account<'info, FanoutMembershipVoucher>,
}

pub fn transfer_shares(ctx: Context<TransferShares>, shares: u64) -> Result<()> {
    let fanout = &mut ctx.accounts.fanout;
    let from_membership_account = &mut ctx.accounts.from_membership_account;
    let to_membership_account = &mut ctx.accounts.to_membership_account;

    if to_membership_account.key() == from_membership_account.key() {
        return Err(HydraError::CannotTransferToSelf.into());
    }

    if from_membership_account.shares < shares {
        return Err(HydraError::InsufficientShares.into());
    }

    if fanout.membership_model != MembershipModel::NFT
        && fanout.membership_model != MembershipModel::Wallet
    {
        return Err(HydraError::TransferNotSupported.into());
    }
    from_membership_account.shares -= shares;
    to_membership_account.shares += shares;
    Ok(())
}
