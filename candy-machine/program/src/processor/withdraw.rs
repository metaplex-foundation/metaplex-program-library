use anchor_lang::prelude::*;

use crate::{cmp_pubkeys, CandyError, CandyMachine};

/// Withdraw SOL from candy machine account.
#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(mut, has_one = authority)]
    candy_machine: Account<'info, CandyMachine>,
    #[account(address = candy_machine.authority)]
    authority: Signer<'info>,
    // > Only if collection
    // CollectionPDA account
}

pub fn handle_withdraw_funds<'info>(ctx: Context<WithdrawFunds<'info>>) -> Result<()> {
    let authority = &ctx.accounts.authority;
    let pay = &ctx.accounts.candy_machine.to_account_info();
    let snapshot: u64 = pay.lamports();

    **pay.lamports.borrow_mut() = 0;

    **authority.lamports.borrow_mut() = authority
        .lamports()
        .checked_add(snapshot)
        .ok_or(CandyError::NumericalOverflowError)?;

    if !ctx.remaining_accounts.is_empty() {
        let seeds = [b"collection".as_ref(), pay.key.as_ref()];
        let pay = &ctx.remaining_accounts[0];
        if !cmp_pubkeys(
            &pay.key(),
            &Pubkey::find_program_address(&seeds, &crate::id()).0,
        ) {
            return err!(CandyError::MismatchedCollectionPDA);
        }
        let snapshot: u64 = pay.lamports();
        **pay.lamports.borrow_mut() = 0;
        **authority.lamports.borrow_mut() = authority
            .lamports()
            .checked_add(snapshot)
            .ok_or(CandyError::NumericalOverflowError)?;
    }

    Ok(())
}
