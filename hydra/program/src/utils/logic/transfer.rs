use crate::{error::HydraError, state::Fanout};
use anchor_lang::prelude::*;

pub fn transfer_from_mint_holding<'info>(
    fanout: &Fanout,
    fanout_authority: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    source: AccountInfo<'info>,
    dest: AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    if amount > 0 {
        let cpi_program = token_program;
        let accounts = anchor_spl::token::Transfer {
            from: source,
            to: dest,
            authority: fanout_authority.clone(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, accounts);

        let seeds = [
            b"fanout-config".as_ref(),
            fanout.name.as_bytes(),
            &[fanout.bump_seed],
        ];
        return anchor_spl::token::transfer(cpi_ctx.with_signer(&[&seeds]), amount);
    }
    Ok(())
}

pub fn transfer_native<'info>(
    source: AccountInfo<'info>,
    dest: AccountInfo<'info>,
    current_snapshot: u64,
    amount: u64,
) -> Result<()> {
    if amount > 0 {
        **source.lamports.borrow_mut() = current_snapshot
            .checked_sub(amount)
            .ok_or(HydraError::NumericalOverflow)?;
        **dest.lamports.borrow_mut() = dest
            .lamports()
            .checked_add(amount)
            .ok_or(HydraError::NumericalOverflow)?;
    }
    Ok(())
}
