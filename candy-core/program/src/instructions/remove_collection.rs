use anchor_lang::prelude::*;
use mpl_token_metadata::{
    instruction::revoke_collection_authority, state::Metadata, state::TokenMetadataAccount,
};
use solana_program::program::invoke;

use crate::{cmp_pubkeys, constants::COLLECTION_SEED, CandyError, CandyMachine};

pub fn remove_collection(ctx: Context<RemoveCollection>) -> Result<()> {
    let mint = ctx.accounts.collection_mint.to_account_info();
    let candy_machine = &mut ctx.accounts.candy_machine;
    if candy_machine.items_redeemed > 0 {
        return err!(CandyError::NoChangingCollectionDuringMint);
    }
    let metadata: Metadata =
        Metadata::from_account_info(&ctx.accounts.collection_metadata.to_account_info())?;
    if !cmp_pubkeys(&metadata.update_authority, &ctx.accounts.update_authority.key()) {
        return err!(CandyError::IncorrectCollectionAuthority);
    };
    if !cmp_pubkeys(&metadata.mint, &mint.key()) {
        return err!(CandyError::MintMismatch);
    }
    let authority_record = ctx.accounts.collection_authority_record.to_account_info();
    let revoke_collection_infos = vec![
        authority_record.clone(),
        ctx.accounts.collection_authority.to_account_info(),
        ctx.accounts.authority.to_account_info(),
        ctx.accounts.collection_metadata.to_account_info(),
        mint.clone(),
    ];
    msg!(
        "Revoking collection authority for {}.",
        ctx.accounts.collection_metadata.key()
    );
    invoke(
        &revoke_collection_authority(
            ctx.accounts.token_metadata_program.key(),
            authority_record.key(),
            ctx.accounts.collection_authority.key(),
            ctx.accounts.authority.key(),
            ctx.accounts.collection_metadata.key(),
            mint.key(),
        ),
        revoke_collection_infos.as_slice(),
    )?;

    candy_machine.collection_mint = None;

    Ok(())
}

/// Set the collection PDA for the candy machine
#[derive(Accounts)]
pub struct RemoveCollection<'info> {
    #[account(mut, has_one = authority, has_one = update_authority)]
    candy_machine: Account<'info, CandyMachine>,
    authority: Signer<'info>,
    /// CHECK: authority can be any account and is not written to or read
    update_authority: UncheckedAccount<'info>,
    /// CHECK: only used as a signer
    #[account(
        seeds = [COLLECTION_SEED.as_bytes(), candy_machine.to_account_info().key.as_ref()],
        bump
    )]
    collection_authority: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    collection_mint: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    collection_metadata: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_authority_record: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(address = mpl_token_metadata::id())]
    token_metadata_program: UncheckedAccount<'info>,
}
