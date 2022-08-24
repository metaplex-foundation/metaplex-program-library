use anchor_lang::prelude::*;
use mpl_token_metadata::{
    assertions::collection::assert_master_edition, instruction::approve_collection_authority,
    state::Metadata, state::TokenMetadataAccount,
};
use solana_program::program::invoke;

use crate::{cmp_pubkeys, constants::COLLECTION_SEED, CandyError, CandyMachine};

pub fn add_collection(ctx: Context<AddCollection>) -> Result<()> {
    let mint = ctx.accounts.collection_mint.to_account_info();
    let metadata: Metadata =
        Metadata::from_account_info(&ctx.accounts.collection_metadata.to_account_info())?;

    if !cmp_pubkeys(&metadata.update_authority, &ctx.accounts.update_authority.key()) {
        return err!(CandyError::IncorrectCollectionAuthority);
    }

    if !cmp_pubkeys(&metadata.mint, &mint.key()) {
        return err!(CandyError::MintMismatch);
    }

    let edition = ctx.accounts.collection_edition.to_account_info();
    let authority_record = ctx.accounts.collection_authority_record.to_account_info();
    let candy_machine = &mut ctx.accounts.candy_machine;

    if candy_machine.items_redeemed > 0 {
        return err!(CandyError::NoChangingCollectionDuringMint);
    }

    // the candy machine authority will become the collection update authority
    // therefore we require that retain_authority is set to true
    if !candy_machine.data.retain_authority {
        return err!(CandyError::CandyCollectionRequiresRetainAuthority);
    }

    assert_master_edition(&metadata, &edition)?;

    if authority_record.data_is_empty() {
        let approve_collection_infos = vec![
            authority_record.clone(),
            ctx.accounts.collection_authority.to_account_info(),
            ctx.accounts.update_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.collection_metadata.to_account_info(),
            mint.clone(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ];

        msg!(
            "Approving collection authority for {} with new authority {}.",
            ctx.accounts.collection_metadata.key(),
            ctx.accounts.collection_authority.key()
        );

        invoke(
            &approve_collection_authority(
                ctx.accounts.token_metadata_program.key(),
                authority_record.key(),
                ctx.accounts.collection_authority.key(),
                ctx.accounts.update_authority.key(),
                ctx.accounts.payer.key(),
                ctx.accounts.collection_metadata.key(),
                *mint.key,
            ),
            approve_collection_infos.as_slice(),
        )?;

        msg!(
            "Successfully approved collection authority for collection mint {}.",
            mint.key()
        );
    }

    candy_machine.collection_mint = Some(mint.key());

    Ok(())
}

/// Set the collection PDA for the candy machine
#[derive(Accounts)]
pub struct AddCollection<'info> {
    #[account(mut, has_one = authority, has_one = update_authority)]
    candy_machine: Account<'info, CandyMachine>,
    // candy machine authority
    authority: Signer<'info>,
    /// CHECK: authority can be any account and is not written to or read
    update_authority: UncheckedAccount<'info>,
    // payer of the transaction
    payer: Signer<'info>,
    #[account(
        seeds = [COLLECTION_SEED.as_bytes(), candy_machine.to_account_info().key.as_ref()],
        bump
    )]
    /// CHECK: account checked in CPI
    collection_authority: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    collection_metadata: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    collection_mint: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    collection_edition: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_authority_record: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(address = mpl_token_metadata::id())]
    token_metadata_program: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}
