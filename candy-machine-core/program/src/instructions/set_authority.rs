use anchor_lang::prelude::*;

use crate::CandyMachine;

pub fn set_authority(ctx: Context<SetAuthority>, new_authority: Pubkey) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;

    candy_machine.authority = new_authority;

    Ok(())
}

/// Sets a new candy machine authority.
#[derive(Accounts)]
pub struct SetAuthority<'info> {
    /// Candy Machine account.
    #[account(mut, has_one = authority)]
    candy_machine: Account<'info, CandyMachine>,

    /// Autority of the candy machine.
    authority: Signer<'info>,
}
