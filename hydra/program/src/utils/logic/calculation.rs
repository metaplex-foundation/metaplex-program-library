use crate::{
    error::{HydraError, OrArithError},
    state::{Fanout, FanoutMembershipMintVoucher, FanoutMembershipVoucher, FanoutMint},
};
use anchor_lang::prelude::*;

pub fn calculate_inflow_change(total_inflow: u64, last_inflow: u64) -> Result<u64> {
    let diff: u64 = total_inflow.checked_sub(last_inflow).or_arith_error()?;
    Ok(diff)
}

pub fn calculate_dist_amount(
    member_shares: u64,
    inflow_diff: u64,
    total_shares: u64,
) -> Result<u64> {
    let member_shares = member_shares as u128;
    let total_shares = total_shares as u128;
    let inflow_diff = inflow_diff as u128;
    let dist_amount = member_shares
        .checked_mul(inflow_diff)
        .or_arith_error()?
        .checked_div(total_shares)
        .or_arith_error()?;
    Ok(dist_amount as u64)
}

pub fn update_fanout_for_add(fanout: &mut Account<Fanout>, shares: u64) -> Result<()> {
    let less_shares = fanout
        .total_available_shares
        .checked_sub(shares)
        .or_arith_error()?;
    fanout.total_members = fanout.total_members.checked_add(1).or_arith_error()?;
    fanout.total_available_shares = less_shares;
    if less_shares.ge(&0) {
        Ok(())
    } else {
        Err(HydraError::InsufficientShares.into())
    }
}

pub fn update_fanout_for_remove(fanout: &mut Account<Fanout>) -> Result<()> {
    fanout.total_members = fanout.total_members.checked_sub(1).or_arith_error()?;
    Ok(())
}

pub fn update_inflow_for_mint(
    fanout: &mut Account<Fanout>,
    fanout_for_mint: &mut FanoutMint,
    current_snapshot: u64,
) -> Result<()> {
    let diff = current_snapshot
        .checked_sub(fanout_for_mint.last_snapshot_amount)
        .or_arith_error()?;
    fanout_for_mint.total_inflow = fanout_for_mint
        .total_inflow
        .checked_add(diff)
        .or_arith_error()?;
    if fanout.total_staked_shares.is_some() && fanout.total_staked_shares.unwrap() > 0 {
        let tss = fanout.total_staked_shares.unwrap();
        let shares_diff = (fanout.total_shares as u64)
            .checked_sub(tss)
            .or_arith_error()?;
        let unstaked_correction = (diff as u128)
            .checked_mul(shares_diff as u128)
            .or_arith_error()?
            .checked_div(tss as u128)
            .or_arith_error()? as u64;
        fanout_for_mint.total_inflow += unstaked_correction;
    }
    fanout_for_mint.last_snapshot_amount = current_snapshot;
    Ok(())
}

pub fn update_inflow(fanout: &mut Fanout, current_snapshot: u64) -> Result<()> {
    let diff = current_snapshot
        .checked_sub(fanout.last_snapshot_amount)
        .or_arith_error()?;
    fanout.total_inflow = fanout.total_inflow.checked_add(diff).or_arith_error()?;
    if fanout.total_staked_shares.is_some() && fanout.total_staked_shares.unwrap() > 0 {
        let tss = fanout.total_staked_shares.unwrap();
        let shares_diff = (fanout.total_shares as u64)
            .checked_sub(tss)
            .or_arith_error()?;
        let unstaked_correction = (diff as u128)
            .checked_mul(shares_diff as u128)
            .or_arith_error()?
            .checked_div(tss as u128)
            .or_arith_error()? as u64;
        fanout.total_inflow += unstaked_correction;
    }
    fanout.last_snapshot_amount = current_snapshot;
    Ok(())
}

pub fn update_snapshot(
    fanout: &mut Account<Fanout>,
    fanout_voucher: &mut Account<FanoutMembershipVoucher>,
    distribution_amount: u64,
) -> Result<()> {
    fanout_voucher.last_inflow = fanout.total_inflow;
    fanout.last_snapshot_amount = fanout
        .last_snapshot_amount
        .checked_sub(distribution_amount)
        .or_arith_error()?;
    Ok(())
}

pub fn update_snapshot_for_mint(
    fanout_mint: &mut FanoutMint,
    fanout_mint_voucher: &mut FanoutMembershipMintVoucher,
    distribution_amount: u64,
) -> Result<()> {
    fanout_mint_voucher.last_inflow = fanout_mint.total_inflow;
    fanout_mint.last_snapshot_amount = fanout_mint
        .last_snapshot_amount
        .checked_sub(distribution_amount)
        .or_arith_error()?;
    Ok(())
}

pub fn current_lamports(
    rent: &Sysvar<Rent>,
    size: usize,
    holding_account_lamports: u64,
) -> Result<u64> {
    let subtract_size = rent.minimum_balance(size).max(1);
    holding_account_lamports
        .checked_sub(subtract_size)
        .ok_or_else(|| HydraError::NumericalOverflow.into())
}
