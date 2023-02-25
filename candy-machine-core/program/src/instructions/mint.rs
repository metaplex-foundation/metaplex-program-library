use anchor_lang::prelude::*;
use solana_program::sysvar;

use super::mint_v2::{process_mint, MintAccounts};
use crate::{constants::AUTHORITY_SEED, utils::*, AccountVersion, CandyError, CandyMachine};

pub fn mint<'info>(ctx: Context<'_, '_, '_, 'info, Mint<'info>>) -> Result<()> {
    if !matches!(ctx.accounts.candy_machine.version, AccountVersion::V1) {
        return err!(CandyError::InvalidAccountVersion);
    }

    let accounts = MintAccounts {
        spl_ata_program: None,
        authority_pda: ctx.accounts.authority_pda.to_account_info(),
        delegate_record: ctx.accounts.collection_authority_record.to_account_info(),
        collection_master_edition: ctx.accounts.collection_master_edition.to_account_info(),
        collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
        collection_mint: ctx.accounts.collection_mint.to_account_info(),
        collection_update_authority: ctx.accounts.collection_update_authority.to_account_info(),
        nft_master_edition: ctx.accounts.nft_master_edition.to_account_info(),
        nft_metadata: ctx.accounts.nft_metadata.to_account_info(),
        nft_mint: ctx.accounts.nft_mint.to_account_info(),
        nft_mint_authority: ctx.accounts.nft_mint_authority.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        recent_slothashes: ctx.accounts.recent_slothashes.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        sysvar_instructions: None,
        token: None,
        token_metadata_program: ctx.accounts.token_metadata_program.to_account_info(),
        spl_token_program: ctx.accounts.token_program.to_account_info(),
        token_record: None,
    };

    process_mint(
        &mut ctx.accounts.candy_machine,
        accounts,
        ctx.bumps["authority_pda"],
    )
}

/// Mint a new NFT.
#[derive(Accounts)]
pub struct Mint<'info> {
    /// Candy machine account.
    #[account(mut, has_one = mint_authority)]
    candy_machine: Box<Account<'info, CandyMachine>>,

    /// Candy machine authority account. This is the account that holds a delegate
    /// to verify an item into the collection.
    ///
    /// CHECK: account constraints checked in account trait
    #[account(mut, seeds = [AUTHORITY_SEED.as_bytes(), candy_machine.key().as_ref()], bump)]
    authority_pda: UncheckedAccount<'info>,

    /// Candy machine mint authority (mint only allowed for the mint_authority).
    mint_authority: Signer<'info>,

    /// Payer for the transaction and account allocation (rent).
    #[account(mut)]
    payer: Signer<'info>,

    /// Mint account of the NFT. The account will be initialized if necessary.
    ///
    /// CHECK: account checked in CPI
    #[account(mut)]
    nft_mint: UncheckedAccount<'info>,

    /// Mint authority of the NFT. In most cases this will be the owner of the NFT.
    nft_mint_authority: Signer<'info>,

    /// Metadata account of the NFT. This account must be uninitialized.
    ///
    /// CHECK: account checked in CPI
    #[account(mut)]
    nft_metadata: UncheckedAccount<'info>,

    /// Master edition account of the NFT. The account will be initialized if necessary.
    ///
    /// CHECK: account checked in CPI
    #[account(mut)]
    nft_master_edition: UncheckedAccount<'info>,

    /// Collection authority record account is either the delegated authority record (legacy)
    /// or a metadata delegate record for the `authority_pda`. The delegate is set when a new collection
    /// is set to the candy machine.
    ///
    /// CHECK: account checked in CPI
    collection_authority_record: UncheckedAccount<'info>,

    /// Mint account of the collection NFT.
    ///
    /// CHECK: account checked in CPI
    collection_mint: UncheckedAccount<'info>,

    /// Metadata account of the collection NFT.
    ///
    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_metadata: UncheckedAccount<'info>,

    /// Master edition account of the collection NFT.
    ///
    /// CHECK: account checked in CPI
    collection_master_edition: UncheckedAccount<'info>,

    /// Update authority of the collection NFT.
    ///
    /// CHECK: account checked in CPI
    collection_update_authority: UncheckedAccount<'info>,

    /// Token Metadata program.
    ///
    /// CHECK: account checked in CPI
    #[account(address = mpl_token_metadata::id())]
    token_metadata_program: UncheckedAccount<'info>,

    /// SPL Token program.
    token_program: Program<'info, Token>,

    /// System program.
    system_program: Program<'info, System>,

    /// SlotHashes sysvar cluster data.
    ///
    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::slot_hashes::id())]
    recent_slothashes: UncheckedAccount<'info>,
}
