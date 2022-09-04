use anchor_lang::prelude::*;
use solana_program::{
    entrypoint::MAX_PERMITTED_DATA_INCREASE, program::invoke, system_instruction,
};

use crate::{
    errors::CandyGuardError,
    state::{CandyGuard, CandyGuardData, DATA_OFFSET},
};

pub fn update(ctx: Context<Update>, data: CandyGuardData) -> Result<()> {
    let account_info = ctx.accounts.candy_guard.to_account_info();

    // check whether we need to grow or shrink the account size or not
    if data.size() != account_info.data_len() {
        // no risk of overflow here since the sizes will range from DATA_OFFSET to 10_000_000
        let difference = data.size() as i64 - account_info.data_len() as i64;
        let snapshot = account_info.lamports();

        if difference > 0 {
            if difference as usize > MAX_PERMITTED_DATA_INCREASE {
                return err!(CandyGuardError::DataIncrementLimitExceeded);
            }

            let lamports_diff = Rent::get()?
                .minimum_balance(data.size())
                .checked_sub(snapshot)
                .ok_or(CandyGuardError::NumericalOverflowError)?;

            msg!("Funding {} lamports for account realloc", lamports_diff);

            invoke(
                &system_instruction::transfer(
                    ctx.accounts.payer.key,
                    account_info.key,
                    lamports_diff,
                ),
                &[
                    ctx.accounts.payer.to_account_info(),
                    account_info.clone(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
        } else {
            let lamports_diff = snapshot
                .checked_sub(Rent::get()?.minimum_balance(data.size()))
                .ok_or(CandyGuardError::NumericalOverflowError)?;

            msg!(
                "Withdrawing {} lamports from account realloc",
                lamports_diff
            );

            **account_info.lamports.borrow_mut() = snapshot - lamports_diff;
            let payer = &ctx.accounts.payer;

            **payer.lamports.borrow_mut() = payer
                .lamports()
                .checked_add(lamports_diff)
                .ok_or(CandyGuardError::NumericalOverflowError)?;
        }

        msg!("Account realloc by {} bytes", difference);
        // changes the account size to fit the size required by the guards
        // this means that the size can grow or shrink
        account_info.realloc(data.size(), false)?;
    }

    // save the guards information to the account data and stores
    // the updated feature flag
    let mut account_data = account_info.data.borrow_mut();
    data.save(&mut account_data[DATA_OFFSET..])?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(data: CandyGuardData)]
pub struct Update<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"candy_guard", candy_guard.base.key().as_ref()],
        bump = candy_guard.bump
    )]
    pub candy_guard: Account<'info, CandyGuard>,
    pub authority: Signer<'info>,
    // Payer for the account resizing.
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
