use std::collections::BTreeMap;

use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::token::Token;

use mpl_candy_machine_core::CandyMachine;

use crate::{
    guards::{CandyGuardError, EvaluationContext},
    state::{CandyGuard, CandyGuardData, DATA_OFFSET},
    utils::cmp_pubkeys,
};

pub fn mint<'info>(
    ctx: Context<'_, '_, '_, 'info, Mint<'info>>,
    creator_bump: u8,
    mint_args: Vec<u8>,
) -> Result<()> {
    let candy_guard = &ctx.accounts.candy_guard;
    let account_info = &candy_guard.to_account_info();
    // loads the active guard set
    let account_data = account_info.data.borrow();
    let guard_set = CandyGuardData::active_set(&account_data[DATA_OFFSET..])?;

    let conditions = guard_set.enabled_conditions();
    let process_error = |error: Error| -> Result<()> {
        if let Some(bot_tax) = &guard_set.bot_tax {
            bot_tax.punish_bots(error, &ctx)?;
            Ok(())
        } else {
            Err(error)
        }
    };

    // evaluation context for this transaction
    let mut evaluation_context = EvaluationContext {
        is_authority: cmp_pubkeys(&candy_guard.authority, &ctx.accounts.payer.key()),
        account_cursor: 0,
        args_cursor: 0,
        is_presale: false,
        indices: BTreeMap::new(),
        lamports: 0,
        amount: 0,
        whitelist: false,
    };

    // validates the required transaction data
    if let Err(error) = validate(&ctx) {
        return process_error(error);
    }

    // validates enabled guards (any error at this point is subject to bot tax)

    for condition in &conditions {
        if let Err(error) =
            condition.validate(&ctx, &mint_args, &guard_set, &mut evaluation_context)
        {
            return process_error(error);
        }
    }

    // performs guard pre-actions (errors might occur, which will cause the transaction to fail)
    // no bot tax at this point since the actions must be reverted in case of an error

    for condition in &conditions {
        condition.pre_actions(&ctx, &mint_args, &guard_set, &mut evaluation_context)?;
    }

    // we are good to go, forward the transaction to the candy machine (if errors occur, the
    // actions are reverted and the trasaction fails)

    cpi_mint(&ctx, creator_bump)?;

    // performs guard post-actions (errors might occur, which will cause the transaction to fail)
    // no bot tax at this point since the actions must be reverted in case of an error

    for condition in &conditions {
        condition.post_actions(&ctx, &mint_args, &guard_set, &mut evaluation_context)?;
    }

    Ok(())
}

/// Performs a validation of the transaction before executing the guards.
fn validate<'info>(ctx: &Context<'_, '_, '_, 'info, Mint<'info>>) -> Result<()> {
    if !cmp_pubkeys(
        &ctx.accounts.collection_mint.key(),
        &ctx.accounts.candy_machine.collection_mint,
    ) {
        return err!(CandyGuardError::CollectionKeyMismatch);
    }
    if !cmp_pubkeys(
        ctx.accounts.collection_metadata.owner,
        &mpl_token_metadata::id(),
    ) {
        return err!(CandyGuardError::IncorrectOwner);
    }

    Ok(())
}

/// Send a mint transaction to the candy machine.
fn cpi_mint<'info>(ctx: &Context<'_, '_, '_, 'info, Mint<'info>>, creator_bump: u8) -> Result<()> {
    let candy_guard = &ctx.accounts.candy_guard;
    // PDA signer for the transaction
    let seeds = [
        b"candy_guard".as_ref(),
        &candy_guard.base.to_bytes(),
        &[candy_guard.bump],
    ];
    let signer = [&seeds[..]];
    let candy_machine_program = ctx.accounts.candy_machine_program.to_account_info();

    // candy machine mint instruction accounts
    let mint_ix = mpl_candy_machine_core::cpi::accounts::Mint {
        candy_machine: ctx.accounts.candy_machine.to_account_info(),
        candy_machine_creator: ctx.accounts.candy_machine_creator.to_account_info(),
        authority: ctx.accounts.candy_guard.to_account_info(),
        update_authority: ctx.accounts.update_authority.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        metadata: ctx.accounts.metadata.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        mint_authority: ctx.accounts.mint_authority.to_account_info(),
        master_edition: ctx.accounts.master_edition.to_account_info(),
        collection_authority_record: ctx.accounts.collection_authority_record.to_account_info(),
        collection_mint: ctx.accounts.collection_mint.to_account_info(),
        collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
        collection_master_edition: ctx.accounts.collection_master_edition.to_account_info(),
        token_metadata_program: ctx.accounts.token_metadata_program.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
        recent_slothashes: ctx.accounts.recent_slothashes.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(candy_machine_program, mint_ix, &signer);

    mpl_candy_machine_core::cpi::mint(cpi_ctx, creator_bump)
}

#[derive(Accounts)]
#[instruction(creator_bump: u8, mint_args: Vec<u8>)]
pub struct Mint<'info> {
    #[account(seeds = [b"candy_guard", candy_guard.base.key().as_ref()], bump)]
    pub candy_guard: Account<'info, CandyGuard>,
    /// CHECK: account constraints checked in account trait
    #[account(address = mpl_candy_machine_core::id())]
    pub candy_machine_program: AccountInfo<'info>,
    #[account(
        mut,
        has_one = update_authority,
        constraint = candy_guard.key() == candy_machine.authority
    )]
    pub candy_machine: Box<Account<'info, CandyMachine>>,
    /// CHECK: authority can be any account and is not written to or read
    pub update_authority: UncheckedAccount<'info>,
    // seeds and bump are not validated by the candy guard, they will be validated
    // by the CPI'd candy machine mint instruction
    /// CHECK: account constraints checked in account trait
    #[account(mut)]
    pub candy_machine_creator: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    // with the following accounts we aren't using anchor macros because they are CPI'd
    // through to token-metadata which will do all the validations we need on them.
    /// CHECK: account checked in CPI
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,
    pub mint_authority: Signer<'info>,
    pub mint_update_authority: Signer<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    pub collection_authority_record: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    pub collection_mint: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    pub collection_master_edition: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(address = mpl_token_metadata::id())]
    pub token_metadata_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::slot_hashes::id())]
    pub recent_slothashes: UncheckedAccount<'info>,
    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::instructions::id())]
    pub instruction_sysvar_account: UncheckedAccount<'info>,
    // remaining accounts:
    // > needed if lamports guard enabled
    // destination
    // > needed if spl_token guard enabled
    // token_account_info
    // transfer_authority_info
    // destination_ata
    // > needed if third_party_signer guard enabled
    // signer
    // > needed if whitelist guard enabled
    // whitelist_token_account
    // > needed if whitelist guard enabled and mode is "BurnEveryTime"
    // whitelist_token_mint
    // whitelist_burn_authority
    // > needed if gatekeeper guard enabled
    // gateway_token
    // > needed if gatekeeper guard enabled and expire_on_use is true
    // gateway program
    // network_expire_feature
    // > needed if nft_payment guard enabled
    // token_account
    // token_metadata
    // > needed if nft_payment guard enabled and burn is true
    // token_edition
    // mint_account
    // mint_collection_metadata
    // > needed if nft_payment guard enabled and burn is false (transfer)
    // transfer_authority
    // destination_ata
}
