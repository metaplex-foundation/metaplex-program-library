use anchor_lang::prelude::*;

use crate::{CandyError, CandyMachine};

pub fn withdraw<'info>(ctx: Context<Withdraw<'info>>) -> Result<()> {
    let authority = &ctx.accounts.authority;
    let candy_machine = &ctx.accounts.candy_machine.to_account_info();
    let snapshot: u64 = candy_machine.lamports();

    **candy_machine.lamports.borrow_mut() = 0;

    **authority.lamports.borrow_mut() = authority
        .lamports()
        .checked_add(snapshot)
        .ok_or(CandyError::NumericalOverflowError)?;

    Ok(())
}

/// Withdraw the rent SOL from the candy machine account.
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, has_one = authority)]
    candy_machine: Account<'info, CandyMachine>,
    #[account(address = candy_machine.authority)]
    authority: Signer<'info>,
}
