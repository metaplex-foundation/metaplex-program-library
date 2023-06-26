use crate::error::HydraError;
use crate::state::{Fanout, FanoutMembershipVoucher, MembershipModel};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(saturation_limit: u64)]
pub struct SetSaturation<'info> {
    pub authority: Signer<'info>,
    /// CHECK: Native Account
    pub member: UncheckedAccount<'info>,
    #[account(
    mut,
    seeds = [b"fanout-config", fanout.name.as_bytes()],
    has_one = authority,
    bump = fanout.bump_seed,
    )]
    pub fanout: Account<'info, Fanout>,
    #[account(
    mut,
    seeds = [b"fanout-membership", fanout.key().as_ref(), member.key().as_ref()],
    bump,
    has_one = fanout,
    )]
    pub membership_account: Account<'info, FanoutMembershipVoucher>,
}

pub fn set_saturation(ctx: Context<SetSaturation>, saturation_limit: u64) -> Result<()> {
    let fanout = &mut ctx.accounts.fanout;

    if fanout.membership_model != MembershipModel::Wallet {
        return Err(HydraError::SaturationNotSupported.into());
    }

    let voucher = &mut ctx.accounts.membership_account;

    voucher.saturation_limit = saturation_limit;

    Ok(())
}
