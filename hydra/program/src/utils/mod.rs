pub mod logic;
pub mod validation;

use crate::{
    error::HydraError,
    state::{FanoutMembershipMintVoucher, FanoutMint, FANOUT_MINT_MEMBERSHIP_VOUCHER_SIZE},
    utils::validation::{assert_derivation, assert_owned_by},
};
use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke_signed, system_instruction},
};
use anchor_spl::token::TokenAccount;
use std::convert::TryInto;

pub fn create_or_allocate_account_raw<'a>(
    program_id: Pubkey,
    new_account_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    size: usize,
    signer_seeds: &[&[u8]],
    new_acct_seeds: &[&[u8]],
) -> Result<()> {
    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    let required_lamports = rent
        .minimum_balance(size)
        .max(1)
        .saturating_sub(new_account_info.lamports());
    if required_lamports > 0 {
        let as_arr = [signer_seeds];

        let seeds: &[&[&[u8]]] = if !signer_seeds.is_empty() {
            &as_arr
        } else {
            &[]
        };

        invoke_signed(
            &system_instruction::transfer(payer_info.key, new_account_info.key, required_lamports),
            &[
                payer_info.clone(),
                new_account_info.clone(),
                system_program_info.clone(),
            ],
            seeds,
        )?;
    }
    let accounts = &[new_account_info.clone(), system_program_info.clone()];
    invoke_signed(
        &system_instruction::allocate(new_account_info.key, size.try_into().unwrap()),
        accounts,
        &[new_acct_seeds],
    )?;
    invoke_signed(
        &system_instruction::assign(new_account_info.key, &program_id),
        accounts,
        &[new_acct_seeds],
    )?;
    Ok(())
}

pub fn parse_fanout_mint(
    fanout_for_mint: &mut UncheckedAccount,
    fanout: &Pubkey,
    fanout_mint: &Pubkey,
) -> Result<FanoutMint> {
    let account_info = fanout_for_mint.to_account_info();
    let fanout_mint_bump = assert_derivation(
        &crate::ID,
        &account_info,
        &[b"fanout-config", fanout.as_ref(), fanout_mint.as_ref()],
        Some(HydraError::InvalidFanoutForMint.into()),
    )?;
    let mut fanout_mint_data: &[u8] = &fanout_for_mint.try_borrow_mut_data()?;
    let fanout_for_mint_object: FanoutMint = FanoutMint::try_deserialize(&mut fanout_mint_data)?;
    if fanout_mint_bump != fanout_for_mint_object.bump_seed {
        msg!("Invalid Fanout For Mint");
        return Err(HydraError::InvalidFanoutForMint.into());
    }
    Ok(fanout_for_mint_object)
}

pub fn parse_token_account(account: &AccountInfo, owner: &Pubkey) -> Result<TokenAccount> {
    let ref_data = account.try_borrow_data()?;
    let mut account_data: &[u8] = &ref_data;
    let account_object = TokenAccount::try_deserialize(&mut account_data)?;
    if &account_object.owner != owner {
        msg!("Token Account has wrong owner");
        return Err(HydraError::IncorrectOwner.into());
    }
    Ok(account_object)
}

pub fn parse_mint_membership_voucher<'info>(
    fanout_for_mint_membership_voucher: &mut UncheckedAccount<'info>,
    rent: &Sysvar<'info, anchor_lang::prelude::Rent>,
    system_program: &Program<'info, System>,
    payer: &anchor_lang::prelude::AccountInfo<'info>,
    membership_key: &Pubkey,
    fanout_for_mint: &Pubkey,
    fanout_mint: &Pubkey,
    fanout: &Pubkey,
    stake_time: i64,
    total_inflow: u64,
) -> Result<FanoutMembershipMintVoucher> {
    let account_info = fanout_for_mint_membership_voucher.to_account_info();
    let mint_membership_voucher_bump = assert_derivation(
        &crate::ID,
        &account_info,
        &[
            b"fanout-membership",
            fanout_for_mint.as_ref(),
            membership_key.as_ref(),
            fanout_mint.as_ref(),
        ],
        Some(HydraError::InvalidMembershipVoucher.into()),
    )?;
    let mint_voucher_empty = fanout_for_mint_membership_voucher.data_is_empty();

    Ok(if mint_voucher_empty {
        create_or_allocate_account_raw(
            crate::ID,
            &account_info,
            &rent.to_account_info(),
            system_program,
            payer,
            FANOUT_MINT_MEMBERSHIP_VOUCHER_SIZE,
            &[],
            &[
                b"fanout-membership",
                fanout_for_mint.as_ref(),
                membership_key.as_ref(),
                fanout_mint.as_ref(),
                &[mint_membership_voucher_bump],
            ],
        )?;
        FanoutMembershipMintVoucher {
            fanout: *fanout,
            fanout_mint: *fanout_mint,
            last_inflow: total_inflow,
            bump_seed: mint_membership_voucher_bump,
            stake_time,
        }
    } else {
        let mut membership_data: &[u8] =
            &fanout_for_mint_membership_voucher.try_borrow_mut_data()?;
        assert_owned_by(fanout_for_mint_membership_voucher, &crate::ID)?;
        let mut membership = FanoutMembershipMintVoucher::try_deserialize(&mut membership_data)?;
        if membership.bump_seed != mint_membership_voucher_bump {
            msg!("Mint Membership Bump Doesnt match");
            return Err(HydraError::InvalidMembershipVoucher.into());
        }
        // If this account was staked before at a different time, clear it out.
        if stake_time != membership.stake_time {
            membership.last_inflow = total_inflow;
            membership.stake_time = stake_time;
        }

        membership
    })
}
