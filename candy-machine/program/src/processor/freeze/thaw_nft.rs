use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::token::{Mint, Token, TokenAccount};
use mpl_token_metadata::instruction::thaw_delegated_account;
use solana_program::program::{invoke, invoke_signed};
use spl_token::instruction::revoke;

use crate::{cmp_pubkeys, CandyError, CandyMachine, FreezePDA};

/// Set the collection PDA for the candy machine
#[derive(Accounts)]
pub struct ThawNFT<'info> {
    #[account(mut, seeds = [FreezePDA::PREFIX.as_bytes(), candy_machine.key().as_ref()], bump, has_one = candy_machine)]
    freeze_pda: Account<'info, FreezePDA>,
    /// CHECK: account could be empty so must be unchecked. Checked in freeze_pda constraint.
    #[account(mut)]
    candy_machine: UncheckedAccount<'info>,
    #[account(mut, has_one = mint, has_one = owner)]
    token_account: Account<'info, TokenAccount>,
    /// CHECK: checked in token_account constraints
    owner: UncheckedAccount<'info>,
    mint: Account<'info, Mint>,
    /// CHECK: account checked in CPI
    edition: UncheckedAccount<'info>,
    #[account(mut)]
    payer: Signer<'info>,
    token_program: Program<'info, Token>,
    /// CHECK: checked in account constraints
    #[account(address = mpl_token_metadata::id())]
    token_metadata_program: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
}

pub fn handle_thaw_nft(ctx: Context<ThawNFT>) -> Result<()> {
    let freeze_pda = &mut ctx.accounts.freeze_pda;
    let candy_machine = &mut ctx.accounts.candy_machine;
    let current_timestamp = Clock::get()?.unix_timestamp;
    let can_thaw = if candy_machine.data_is_empty() {
        // shouldn't be possible to get into this state with NFTs still not frozen
        true
    } else {
        let candy_struct: Account<CandyMachine> =
            Account::try_from(&candy_machine.to_account_info())?;
        freeze_pda.thaw_eligible(current_timestamp, &candy_struct)
    };
    msg!("Can thaw: {}", can_thaw);
    if !can_thaw {
        return err!(CandyError::InvalidThawNft);
    }
    let token_account = &ctx.accounts.token_account;
    let mint = &ctx.accounts.mint;
    let edition = &ctx.accounts.edition;
    let payer = &ctx.accounts.payer;
    let owner = &ctx.accounts.owner;
    let token_program = &ctx.accounts.token_program;
    let token_metadata_program = &ctx.accounts.token_metadata_program;
    let freeze_seeds = [
        FreezePDA::PREFIX.as_bytes(),
        candy_machine.key.as_ref(),
        &[*ctx.bumps.get("freeze_pda").unwrap()],
    ];
    if token_account.is_frozen() {
        msg!("Token account is frozen! Now attempting to thaw!");
        invoke_signed(
            &thaw_delegated_account(
                mpl_token_metadata::ID,
                freeze_pda.key(),
                token_account.key(),
                edition.key(),
                mint.key(),
            ),
            &[
                freeze_pda.to_account_info(),
                token_account.to_account_info(),
                edition.to_account_info(),
                mint.to_account_info(),
                token_program.to_account_info(),
                token_metadata_program.to_account_info(),
            ],
            &[&freeze_seeds],
        )?;
        if freeze_pda.freeze_fee > 0 && freeze_pda.frozen_count > 0 {
            transfer(
                CpiContext::new(
                    ctx.accounts.system_program.to_account_info(),
                    Transfer {
                        from: freeze_pda.to_account_info(),
                        to: payer.to_account_info(),
                    },
                ),
                freeze_pda.freeze_fee,
            )?;
        }
        // if everything is correct, this saturating sub shouldn't be needed.
        // Just an extra precaution to allow unfreezing if something unexpected were to
        // happen to the freeze count to allow everyone to still unfreeze
        freeze_pda.frozen_count = freeze_pda.frozen_count.saturating_sub(1);
    } else {
        msg!("Token account is not frozen!");
    }
    if cmp_pubkeys(&payer.key(), &owner.key()) {
        msg!("Revoking authority");
        invoke(
            &revoke(&spl_token::ID, &token_account.key(), &payer.key(), &[])?,
            &[token_account.to_account_info(), payer.to_account_info()],
        )?;
    } else {
        msg!("Cannot revoke delegate authority: token account owner is not signer. Re-run as owner to revoke or just call revoke manually.");
    }
    Ok(())
}
