use super::arg::AddMemberArgs;

use crate::state::{Fanout, FanoutMembershipVoucher, FANOUT_MEMBERSHIP_VOUCHER_SIZE};
use crate::utils::logic::calculation::*;
use crate::utils::validation::{assert_membership_model, assert_owned_by, assert_valid_metadata};
use crate::MembershipModel;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

#[derive(Accounts)]
#[instruction(args: AddMemberArgs)]
pub struct AddMemberWithNFT<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
    mut,
    seeds = [b"fanout-config", fanout.name.as_bytes()],
    has_one= authority,
    bump = fanout.bump_seed,
    )]
    pub fanout: Account<'info, Fanout>,
    #[account(
    init,
    space = FANOUT_MEMBERSHIP_VOUCHER_SIZE,
    seeds = [b"fanout-membership", fanout.key().as_ref(), mint.key().as_ref()],
    bump,
    payer = authority
    )]
    pub membership_account: Account<'info, FanoutMembershipVoucher>,
    pub mint: Account<'info, Mint>,
    /// CHECK: Checked in program
    pub metadata: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
}
pub fn add_member_nft(ctx: Context<AddMemberWithNFT>, args: AddMemberArgs) -> Result<()> {
    let fanout = &mut ctx.accounts.fanout;
    let membership_account = &mut ctx.accounts.membership_account;
    let metadata = &ctx.accounts.metadata;
    let mint = &ctx.accounts.mint;
    assert_owned_by(metadata, &mpl_token_metadata::id())?;
    assert_membership_model(fanout, MembershipModel::NFT)?;
    assert_valid_metadata(metadata, &mint.to_account_info())?;
    update_fanout_for_add(fanout, args.shares)?;
    membership_account.membership_key = ctx.accounts.mint.to_account_info().key();
    membership_account.shares = args.shares;
    membership_account.bump_seed = *ctx.bumps.get("membership_account").unwrap();
    membership_account.fanout = fanout.key();
    Ok(())
}
