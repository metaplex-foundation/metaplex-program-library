use anchor_lang::prelude::*;
use mpl_candy_machine_core::CandyMachine;

use crate::state::CandyGuard;

pub fn unwrap(ctx: Context<Unwrap>) -> Result<()> {
    let candy_machine_program = ctx.accounts.candy_machine_program.to_account_info();
    let candy_guard = &ctx.accounts.candy_guard;
    // PDA is the signer of the CPI
    let seeds = [
        b"candy_guard".as_ref(),
        &candy_guard.base.to_bytes(),
        &[candy_guard.bump],
    ];
    let signer = [&seeds[..]];

    let update_ix = mpl_candy_machine_core::cpi::accounts::SetAuthority {
        candy_machine: ctx.accounts.candy_machine.to_account_info(),
        authority: candy_guard.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(candy_machine_program, update_ix, &signer);
    // candy machine update_authority CPI
    mpl_candy_machine_core::cpi::set_authority(
        cpi_ctx,
        ctx.accounts.authority.key(),
        ctx.accounts.candy_machine.update_authority,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct Unwrap<'info> {
    #[account(has_one = authority)]
    pub candy_guard: Account<'info, CandyGuard>,
    #[account(mut, constraint = candy_guard.key() == candy_machine.authority)]
    pub candy_machine: Account<'info, CandyMachine>,
    /// CHECK: account constraints checked in account trait
    #[account(address = mpl_candy_machine_core::id())]
    pub candy_machine_program: AccountInfo<'info>,
    pub authority: Signer<'info>,
}
