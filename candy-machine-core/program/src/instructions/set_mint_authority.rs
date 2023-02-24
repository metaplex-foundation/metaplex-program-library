use anchor_lang::prelude::*;

use crate::CandyMachine;

pub fn set_mint_authority(ctx: Context<SetMintAuthority>) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;

    candy_machine.mint_authority = ctx.accounts.mint_authority.key();

    Ok(())
}

/// Sets a new candy machine authority.
#[derive(Accounts)]
pub struct SetMintAuthority<'info> {
    /// Candy Machine account.
    #[account(mut, has_one = authority)]
    candy_machine: Account<'info, CandyMachine>,

    /// Candy Machine authority
    authority: Signer<'info>,

    /// New candy machine authority
    mint_authority: Signer<'info>,
}
