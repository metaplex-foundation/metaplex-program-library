use crate::state::{Fanout, FanoutMembershipVoucher, HOLDING_ACCOUNT_SIZE};
use crate::utils::logic::calculation::*;
use crate::utils::logic::transfer::{transfer_from_mint_holding, transfer_native};
use crate::utils::parse_fanout_mint;
use crate::utils::validation::*;
use crate::utils::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

pub fn distribute_native<'info>(
    holding_account: &mut UncheckedAccount<'info>,
    fanout: &mut Account<'info, Fanout>,
    membership_voucher: &mut Account<'info, FanoutMembershipVoucher>,
    member: UncheckedAccount<'info>,
    rent: Sysvar<'info, anchor_lang::prelude::Rent>,
) -> Result<()> {
    let total_shares = fanout.total_shares as u64;
    if holding_account.key() != fanout.account_key {
        return Err(HydraError::InvalidHoldingAccount.into());
    }
    let current_snapshot = holding_account.lamports();
    let current_snapshot_less_min =
        current_lamports(&rent, HOLDING_ACCOUNT_SIZE, current_snapshot)?;
    update_inflow(fanout, current_snapshot_less_min)?;
    let inflow_diff = calculate_inflow_change(fanout.total_inflow, membership_voucher.last_inflow)?;
    let shares = membership_voucher.shares as u64;
    let dif_dist = calculate_dist_amount(shares, inflow_diff, total_shares)?;
    update_snapshot(fanout, membership_voucher, dif_dist)?;
    membership_voucher.total_inflow = membership_voucher
        .total_inflow
        .checked_add(dif_dist)
        .ok_or(HydraError::NumericalOverflow)?;
    transfer_native(
        holding_account.to_account_info(),
        member.to_account_info(),
        current_snapshot,
        dif_dist,
    )
}

pub fn distribute_mint<'info>(
    fanout_mint: Account<'info, Mint>,
    fanout_for_mint: &mut UncheckedAccount<'info>,
    fanout_for_mint_membership_voucher: &mut UncheckedAccount<'info>,
    fanout_mint_member_token_account: &mut UncheckedAccount<'info>,
    holding_account: &mut UncheckedAccount<'info>,
    fanout: &mut Account<'info, Fanout>,
    membership_voucher: &mut Account<'info, FanoutMembershipVoucher>,
    rent: Sysvar<'info, anchor_lang::prelude::Rent>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    payer: AccountInfo<'info>,
    member: UncheckedAccount<'info>,
    membership_key: &Pubkey,
) -> Result<()> {
    msg!("Distribute For Mint");
    let mint = &fanout_mint;
    let fanout_for_mint_membership_voucher_unchecked = fanout_for_mint_membership_voucher;
    let fanout_mint_member_token_account_info = fanout_mint_member_token_account.to_account_info();
    let fanout_for_mint = fanout_for_mint;
    let total_shares = fanout.total_shares as u64;
    assert_owned_by(fanout_for_mint, &crate::ID)?;
    assert_owned_by(&fanout_mint_member_token_account_info, &Token::id())?;
    assert_owned_by(holding_account, &anchor_spl::token::Token::id())?;
    assert_ata(
        &holding_account.to_account_info(),
        &fanout.key(),
        &fanout_mint.key(),
        Some(HydraError::HoldingAccountMustBeAnATA.into()),
    )?;
    let fanout_for_mint_object =
        &mut parse_fanout_mint(fanout_for_mint, &fanout.key(), &mint.key())?;
    if holding_account.key() != fanout_for_mint_object.token_account {
        return Err(HydraError::InvalidHoldingAccount.into());
    }
    if fanout_for_mint_object.mint != mint.to_account_info().key() {
        return Err(HydraError::MintDoesNotMatch.into());
    }
    let fanout_for_mint_membership_voucher = &mut parse_mint_membership_voucher(
        fanout_for_mint_membership_voucher_unchecked,
        &rent,
        &system_program,
        &payer.to_account_info(),
        membership_key,
        &fanout_for_mint.key(),
        &mint.key(),
        &fanout.key(),
    )?;
    let holding_account_ata = parse_token_account(holding_account, &fanout.key())?;
    parse_token_account(&fanout_mint_member_token_account_info, &member.key())?;

    let current_snapshot = holding_account_ata.amount;
    update_inflow_for_mint(fanout, fanout_for_mint_object, current_snapshot)?;
    let inflow_diff = calculate_inflow_change(
        fanout_for_mint_object.total_inflow,
        fanout_for_mint_membership_voucher.last_inflow,
    )?;
    let shares = membership_voucher.shares as u64;
    let dif_dist = calculate_dist_amount(shares, inflow_diff, total_shares)?;
    update_snapshot_for_mint(
        fanout_for_mint_object,
        fanout_for_mint_membership_voucher,
        dif_dist,
    )?;

    let mut fanout_for_mint_membership_voucher_data: &mut [u8] =
        &mut fanout_for_mint_membership_voucher_unchecked.try_borrow_mut_data()?;
    let mut fanout_for_mint_data: &mut [u8] = &mut fanout_for_mint.try_borrow_mut_data()?;

    fanout_for_mint_membership_voucher
        .try_serialize(&mut fanout_for_mint_membership_voucher_data)?;
    fanout_for_mint_object.try_serialize(&mut fanout_for_mint_data)?;
    transfer_from_mint_holding(
        fanout,
        fanout.to_account_info(),
        token_program.to_account_info(),
        holding_account.to_account_info(),
        fanout_mint_member_token_account_info,
        dif_dist,
    )
}
