use anchor_lang::{prelude::*, AccountsClose};

use crate::{
    cmp_pubkeys,
    constants::{FREEZE_FEATURE_INDEX, FREEZE_LOCK_FEATURE_INDEX},
    is_feature_active, CandyError, CandyMachine, CollectionPDA,
};

/// Withdraw SOL from candy machine account.
#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(mut, close = authority, has_one = authority)]
    candy_machine: Account<'info, CandyMachine>,
    #[account(mut, address = candy_machine.authority)]
    authority: Signer<'info>,
    // > Only if collection
    // CollectionPDA account
}

pub fn handle_withdraw_funds<'info>(
    ctx: Context<'_, '_, '_, 'info, WithdrawFunds<'info>>,
) -> Result<()> {
    let authority = &ctx.accounts.authority;
    let candy_machine = &ctx.accounts.candy_machine;
    if is_feature_active(&candy_machine.data.uuid, FREEZE_FEATURE_INDEX) {
        return err!(CandyError::NoWithdrawWithFreeze);
    }
    if is_feature_active(&candy_machine.data.uuid, FREEZE_LOCK_FEATURE_INDEX) {
        return err!(CandyError::NoWithdrawWithFrozenFunds);
    }

    if !ctx.remaining_accounts.is_empty() {
        let candy_key = candy_machine.key();
        let seeds = [CollectionPDA::PREFIX.as_bytes(), candy_key.as_ref()];
        let collection_pda = &ctx.remaining_accounts[0];
        if !cmp_pubkeys(
            &collection_pda.key(),
            &Pubkey::find_program_address(&seeds, &crate::id()).0,
        ) {
            return err!(CandyError::MismatchedCollectionPDA);
        }
        let collection_pda: Account<CollectionPDA> =
            Account::try_from(&collection_pda.to_account_info())?;
        collection_pda.close(authority.to_account_info())?;
    }

    Ok(())
}
