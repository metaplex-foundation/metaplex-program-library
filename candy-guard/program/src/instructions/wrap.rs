use anchor_lang::prelude::*;
use mpl_candy_machine_core::CandyMachine;

use crate::state::CandyGuard;

pub fn wrap(ctx: Context<Wrap>) -> Result<()> {
    let candy_machine_program = ctx.accounts.candy_machine_program.to_account_info();
    let update_ix = mpl_candy_machine_core::cpi::accounts::SetAuthority {
        candy_machine: ctx.accounts.candy_machine.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(candy_machine_program, update_ix);
    // candy machine update_authority CPI
    mpl_candy_machine_core::cpi::set_authority(
        cpi_ctx,
        ctx.accounts.candy_guard.key(),
        ctx.accounts.candy_machine.update_authority,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct Wrap<'info> {
    #[account(has_one = authority)]
    pub candy_guard: Account<'info, CandyGuard>,
    #[account(mut, has_one = authority, owner = mpl_candy_machine_core::id())]
    pub candy_machine: Account<'info, CandyMachine>,
    /// CHECK: account constraints checked in account trait
    #[account(address = mpl_candy_machine_core::id())]
    pub candy_machine_program: AccountInfo<'info>,
    pub authority: Signer<'info>,
}
