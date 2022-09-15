use anchor_lang::prelude::*;

use crate::CandyMachine;

pub fn withdraw<'info>(_ctx: Context<Withdraw<'info>>) -> Result<()> {
    Ok(())
}

/// Withdraw the rent SOL from the candy machine account.
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, close=authority, has_one = authority)]
    candy_machine: Account<'info, CandyMachine>,
    #[account(mut)]
    authority: Signer<'info>,
}
