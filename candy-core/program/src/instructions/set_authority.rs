use anchor_lang::prelude::*;

use crate::{errors::CandyError, utils::cmp_pubkeys, CandyMachine};

pub fn set_authority(
    ctx: Context<SetAuthority>,
    new_authority: Pubkey,
    new_update_authority: Pubkey,
) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;

    // TODO: handle this
    if !cmp_pubkeys(&new_update_authority, &candy_machine.update_authority) {
        return err!(CandyError::CannotChangeUpdateAuthority);
    }

    candy_machine.authority = new_authority;
    candy_machine.update_authority = new_update_authority;

    Ok(())
}

#[derive(Accounts)]
pub struct SetAuthority<'info> {
    #[account(mut, has_one = authority)]
    candy_machine: Account<'info, CandyMachine>,
    authority: Signer<'info>,
}
