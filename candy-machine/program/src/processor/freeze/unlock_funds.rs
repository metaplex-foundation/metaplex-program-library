use anchor_lang::prelude::*;
use anchor_spl::token::{close_account, CloseAccount, Token};

use crate::{
    assert_is_ata,
    constants::{FREEZE_FEATURE_INDEX, FREEZE_LOCK_FEATURE_INDEX},
    remove_feature_flag, spl_token_transfer, CandyError, CandyMachine, FreezePDA,
    TokenTransferParams,
};

/// Unlocks the funds from mint stuck in the FreezePDA
#[derive(Accounts)]
pub struct UnlockFunds<'info> {
    #[account(mut, has_one = authority, has_one = wallet)]
    candy_machine: Account<'info, CandyMachine>,
    /// CHECK: wallet is the treasure account of the candy_machine
    #[account(mut)]
    wallet: UncheckedAccount<'info>,
    #[account(mut)]
    authority: Signer<'info>,
    #[account(mut, close = wallet, seeds = [FreezePDA::PREFIX.as_bytes(), candy_machine.to_account_info().key.as_ref()], bump)]
    freeze_pda: Account<'info, FreezePDA>,
    system_program: Program<'info, System>,
    // > Only needed if candy machine has a mint set
    // token_program
    // > Only needed if candy machine has a mint set
    // freeze_ata
    // > Only needed if candy machine has a mint set
    // destination_ata
}

pub fn handle_unlock_funds<'info>(
    ctx: Context<'_, '_, '_, 'info, UnlockFunds<'info>>,
) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;
    let freeze_pda = &mut ctx.accounts.freeze_pda;
    let authority = &mut ctx.accounts.authority;
    if freeze_pda.frozen_count > 0 {
        return err!(CandyError::NoUnlockWithNFTsStillFrozen);
    }

    if !freeze_pda.allow_thaw {
        freeze_pda.allow_thaw = true;
    }
    if let Some(mint) = &candy_machine.token_mint {
        if ctx.remaining_accounts.len() != 3 {
            return err!(CandyError::MissingRemoveFreezeTokenAccounts);
        }
        let token_program = &ctx.remaining_accounts[0];
        require_keys_eq!(token_program.key(), Token::id());

        let freeze_ata_info = &ctx.remaining_accounts[1];
        let freeze_ata = assert_is_ata(freeze_ata_info, &freeze_pda.key(), mint)?;

        let destination_ata = &ctx.remaining_accounts[2];
        require_keys_neq!(
            freeze_ata_info.key(),
            destination_ata.key(),
            CandyError::InvalidFreezeWithdrawTokenAddress
        );

        let candy_key = candy_machine.key();
        let freeze_seeds = [
            FreezePDA::PREFIX.as_bytes(),
            candy_key.as_ref(),
            &[*ctx.bumps.get("freeze_pda").unwrap()],
        ];
        spl_token_transfer(TokenTransferParams {
            source: freeze_ata_info.to_account_info(),
            destination: destination_ata.to_account_info(),
            authority: freeze_pda.to_account_info(),
            authority_signer_seeds: &freeze_seeds,
            token_program: token_program.to_account_info(),
            amount: freeze_ata.amount,
        })?;

        close_account(CpiContext::new_with_signer(
            token_program.to_account_info(),
            CloseAccount {
                account: freeze_ata_info.to_account_info(),
                destination: authority.to_account_info(),
                authority: freeze_pda.to_account_info(),
            },
            &[&freeze_seeds],
        ))?;
    }

    remove_feature_flag(&mut candy_machine.data.uuid, FREEZE_FEATURE_INDEX);
    remove_feature_flag(&mut candy_machine.data.uuid, FREEZE_LOCK_FEATURE_INDEX);
    Ok(())
}
