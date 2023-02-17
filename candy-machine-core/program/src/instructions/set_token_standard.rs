use anchor_lang::{prelude::*, solana_program::sysvar};
use mpl_token_metadata::state::{Metadata, TokenMetadataAccount, TokenStandard};

use crate::{
    approve_collection_authority_helper, approve_metadata_delegate, cmp_pubkeys,
    constants::AUTHORITY_SEED, revoke_collection_authority_helper, revoke_metadata_delegate,
    ApproveCollectionAuthorityHelperAccounts, ApproveMetadataDelegateHelperAccounts, CandyError,
    CandyMachine, RevokeCollectionAuthorityHelperAccounts, RevokeMetadataDelegateHelperAccounts,
    PNFT_FEATURE,
};

pub fn set_token_standard(ctx: Context<SetTokenStandard>, token_standard: u8) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;

    let collection_metadata_info = &ctx.accounts.collection_metadata;
    let collection_metadata: Metadata =
        Metadata::from_account_info(&collection_metadata_info.to_account_info())?;
    // check that the update authority matches the collection update authority
    if !cmp_pubkeys(&collection_metadata.mint, ctx.accounts.collection_mint.key) {
        return err!(CandyError::MintMismatch);
    }

    if token_standard == TokenStandard::ProgrammableNonFungible as u8
        && !candy_machine.is_enabled(PNFT_FEATURE)
    {
        // enables minting pNFTs
        candy_machine.enable_feature(PNFT_FEATURE);

        // revoking the legacy collection authority

        let collection_authority_record = ctx
            .accounts
            .collection_authority_record
            .as_ref()
            .ok_or(CandyError::MissingCollectionAuthorityRecord)?;

        let revoke_accounts = RevokeCollectionAuthorityHelperAccounts {
            authority_pda: ctx.accounts.authority_pda.to_account_info(),
            collection_authority_record: collection_authority_record.to_account_info(),
            collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
            collection_mint: ctx.accounts.collection_mint.to_account_info(),
            token_metadata_program: ctx.accounts.token_metadata_program.to_account_info(),
        };

        revoke_collection_authority_helper(
            revoke_accounts,
            candy_machine.key(),
            *ctx.bumps.get("authority_pda").unwrap(),
        )?;

        // approve a new metadata delegate

        let delegate_record = ctx
            .accounts
            .delegate_record
            .as_ref()
            .ok_or(CandyError::MissingMetadataDelegateRecord)?;

        let delegate_accounts = ApproveMetadataDelegateHelperAccounts {
            authority_pda: ctx.accounts.authority_pda.to_account_info(),
            collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
            collection_mint: ctx.accounts.collection_mint.to_account_info(),
            collection_update_authority: ctx.accounts.collection_update_authority.to_account_info(),
            delegate_record: delegate_record.to_account_info(),
            payer: ctx.accounts.payer.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            sysvar_instructions: ctx.accounts.sysvar_instructions.to_account_info(),
            authorization_rules_program: ctx
                .accounts
                .authorization_rules_program
                .as_ref()
                .map(|authorization_rules_program| authorization_rules_program.to_account_info()),
            authorization_rules: ctx
                .accounts
                .authorization_rules
                .as_ref()
                .map(|authorization_rules| authorization_rules.to_account_info()),
        };

        approve_metadata_delegate(delegate_accounts)
    } else if token_standard == TokenStandard::NonFungible as u8
        && candy_machine.is_enabled(PNFT_FEATURE)
    {
        // disables minting pNFTs
        candy_machine.disable_feature(PNFT_FEATURE);

        // revoking the delegate

        let delegate_record = ctx
            .accounts
            .delegate_record
            .as_ref()
            .ok_or(CandyError::MissingMetadataDelegateRecord)?;

        let revoke_accounts = RevokeMetadataDelegateHelperAccounts {
            authority_pda: ctx.accounts.authority_pda.to_account_info(),
            collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
            collection_mint: ctx.accounts.collection_mint.to_account_info(),
            collection_update_authority: ctx.accounts.collection_update_authority.to_account_info(),
            delegate_record: delegate_record.to_account_info(),
            payer: ctx.accounts.payer.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            sysvar_instructions: ctx.accounts.sysvar_instructions.to_account_info(),
            authorization_rules_program: ctx
                .accounts
                .authorization_rules_program
                .as_ref()
                .map(|authorization_rules_program| authorization_rules_program.to_account_info()),
            authorization_rules: ctx
                .accounts
                .authorization_rules
                .as_ref()
                .map(|authorization_rules| authorization_rules.to_account_info()),
        };

        revoke_metadata_delegate(revoke_accounts)?;

        // setting a legacy collection authority

        let collection_authority_record = ctx
            .accounts
            .collection_authority_record
            .as_ref()
            .ok_or(CandyError::MissingCollectionAuthorityRecord)?;

        let approve_accounts = ApproveCollectionAuthorityHelperAccounts {
            payer: ctx.accounts.payer.to_account_info(),
            authority_pda: ctx.accounts.authority_pda.to_account_info(),
            collection_mint: ctx.accounts.collection_mint.to_account_info(),
            collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
            collection_authority_record: collection_authority_record.to_account_info(),
            token_metadata_program: ctx.accounts.token_metadata_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            collection_update_authority: ctx.accounts.collection_update_authority.to_account_info(),
        };

        approve_collection_authority_helper(approve_accounts)
    } else {
        err!(CandyError::InvalidTokenStandard)
    }
}

#[derive(Accounts)]
#[instruction(token_standard: u8)]
pub struct SetTokenStandard<'info> {
    #[account(mut, has_one = authority, has_one = collection_mint)]
    candy_machine: Account<'info, CandyMachine>,

    // candy machine authority
    authority: Signer<'info>,

    /// CHECK: account checked in CPI
    #[account(
        mut, seeds = [AUTHORITY_SEED.as_bytes(), candy_machine.to_account_info().key.as_ref()],
        bump
    )]
    authority_pda: UncheckedAccount<'info>,

    // payer of the transaction
    payer: Signer<'info>,

    #[account(mut)]
    delegate_record: Option<UncheckedAccount<'info>>,

    /// CHECK: account checked in CPI
    collection_mint: UncheckedAccount<'info>,

    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_metadata: UncheckedAccount<'info>,

    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_authority_record: Option<UncheckedAccount<'info>>,

    #[account(mut)]
    collection_update_authority: Signer<'info>,

    /// CHECK: account checked in CPI
    #[account(address = mpl_token_metadata::id())]
    token_metadata_program: UncheckedAccount<'info>,

    system_program: Program<'info, System>,

    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::instructions::id())]
    sysvar_instructions: UncheckedAccount<'info>,

    /// CHECK: account checked in CPI
    #[account(address = mpl_token_auth_rules::id())]
    authorization_rules_program: Option<UncheckedAccount<'info>>,

    /// CHECK: account constraints checked in account trait
    #[account(owner = mpl_token_auth_rules::id())]
    authorization_rules: Option<UncheckedAccount<'info>>,
}
