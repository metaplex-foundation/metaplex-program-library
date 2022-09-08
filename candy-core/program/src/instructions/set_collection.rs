use anchor_lang::prelude::*;
use mpl_token_metadata::{
    assertions::collection::assert_master_edition, instruction::approve_collection_authority,
    state::Metadata, state::TokenMetadataAccount,
};
use solana_program::program::invoke;

use crate::{cmp_pubkeys, constants::COLLECTION_SEED, CandyError, CandyMachine};

pub fn set_collection(ctx: Context<SetCollection>) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;

    if candy_machine.items_redeemed > 0 {
        return err!(CandyError::NoChangingCollectionDuringMint);
    }
    candy_machine.collection_mint = ctx.accounts.collection_mint.key();

    let set_collection_helper_accounts = SetCollectionHelperAccounts {
        payer: ctx.accounts.payer.to_account_info(),
        update_authority: ctx.accounts.update_authority.to_account_info(),
        collection_mint: ctx.accounts.collection_mint.to_account_info(),
        collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
        collection_master_edition: ctx.accounts.collection_master_edition.to_account_info(),
        collection_authority_record: ctx.accounts.collection_authority_record.to_account_info(),
        token_metadata_program: ctx.accounts.token_metadata_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };

    set_collection_helper(set_collection_helper_accounts)?;

    Ok(())
}

pub fn set_collection_helper(accounts: SetCollectionHelperAccounts) -> Result<()> {
    let metadata: Metadata =
        Metadata::from_account_info(&accounts.collection_metadata.to_account_info())?;

    if !cmp_pubkeys(&metadata.update_authority, &accounts.update_authority.key()) {
        return err!(CandyError::IncorrectCollectionAuthority);
    }

    if !cmp_pubkeys(&metadata.mint, &accounts.collection_mint.key()) {
        return err!(CandyError::MintMismatch);
    }

    let edition = accounts.collection_master_edition.to_account_info();
    let authority_record = accounts.collection_authority_record.to_account_info();

    assert_master_edition(&metadata, &edition)?;

    if authority_record.data_is_empty() {
        let approve_collection_infos = vec![
            authority_record.clone(),
            accounts.update_authority.to_account_info(),
            accounts.update_authority.to_account_info(),
            accounts.payer.to_account_info(),
            accounts.collection_metadata.to_account_info(),
            accounts.collection_mint.to_account_info(),
            accounts.system_program.to_account_info(),
            accounts.rent.to_account_info(),
        ];

        invoke(
            &approve_collection_authority(
                accounts.token_metadata_program.key(),
                authority_record.key(),
                accounts.update_authority.key(),
                accounts.update_authority.key(),
                accounts.payer.key(),
                accounts.collection_metadata.key(),
                accounts.collection_mint.key(),
            ),
            approve_collection_infos.as_slice(),
        )?;
    }

    Ok(())
}

pub struct SetCollectionHelperAccounts<'info> {
    /// CHECK:
    pub payer: AccountInfo<'info>,
    /// CHECK:
    pub update_authority: AccountInfo<'info>,
    /// CHECK:
    pub collection_mint: AccountInfo<'info>,
    /// CHECK:
    pub collection_metadata: AccountInfo<'info>,
    /// CHECK:
    pub collection_master_edition: AccountInfo<'info>,
    /// CHECK:
    pub collection_authority_record: AccountInfo<'info>,
    /// CHECK:
    pub token_metadata_program: AccountInfo<'info>,
    /// CHECK:
    pub system_program: AccountInfo<'info>,
    /// CHECK:
    pub rent: AccountInfo<'info>,
}

/// Set the collection PDA for the candy machine
#[derive(Accounts)]
pub struct SetCollection<'info> {
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
    collection_metadata: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    collection_mint: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    collection_master_edition: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_authority_record: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(address = mpl_token_metadata::id())]
    token_metadata_program: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}
