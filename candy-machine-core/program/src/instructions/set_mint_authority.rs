use anchor_lang::prelude::*;

use crate::CandyMachine;

pub fn set_mint_authority(ctx: Context<SetMintAuthority>) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;

    candy_machine.mint_authority = ctx.accounts.mint_authority.key();

    Ok(())
}

#[derive(Accounts)]
pub struct SetMintAuthority<'info> {
    #[account(mut, has_one = authority)]
    candy_machine: Account<'info, CandyMachine>,
    // candy machine authority
    authority: Signer<'info>,
    // candy machine mint authority to set
    mint_authority: Signer<'info>,
}
