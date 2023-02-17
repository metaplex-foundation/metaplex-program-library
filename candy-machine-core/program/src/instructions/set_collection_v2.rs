use anchor_lang::{prelude::*, solana_program::sysvar};

use crate::{
    approve_collection_authority_helper, approve_metadata_delegate, cmp_pubkeys,
    constants::AUTHORITY_SEED, revoke_collection_authority_helper, revoke_metadata_delegate,
    ApproveCollectionAuthorityHelperAccounts, ApproveMetadataDelegateHelperAccounts, CandyError,
    CandyMachine, RevokeCollectionAuthorityHelperAccounts, RevokeMetadataDelegateHelperAccounts,
    PNFT_FEATURE,
};

pub fn set_collection_v2(ctx: Context<SetCollectionV2>) -> Result<()> {
    let accounts = ctx.accounts;
    let candy_machine = &mut accounts.candy_machine;

    if candy_machine.items_redeemed > 0 {
        return err!(CandyError::NoChangingCollectionDuringMint);
    } else if !cmp_pubkeys(accounts.collection_mint.key, &candy_machine.collection_mint) {
        return err!(CandyError::MintMismatch);
    }

    candy_machine.collection_mint = accounts.new_collection_mint.key();

    if candy_machine.is_enabled(PNFT_FEATURE) {
        // revoking the delegate

        let delegate_record = accounts
            .delegate_record
            .as_ref()
            .ok_or(CandyError::MissingMetadataDelegateRecord)?;

        let revoke_accounts = RevokeMetadataDelegateHelperAccounts {
            authority_pda: accounts.authority_pda.to_account_info(),
            collection_metadata: accounts.collection_metadata.to_account_info(),
            collection_mint: accounts.collection_mint.to_account_info(),
            collection_update_authority: accounts.collection_update_authority.to_account_info(),
            delegate_record: delegate_record.to_account_info(),
            payer: accounts.payer.to_account_info(),
            system_program: accounts.system_program.to_account_info(),
            sysvar_instructions: accounts.sysvar_instructions.to_account_info(),
            authorization_rules_program: accounts
                .authorization_rules_program
                .as_ref()
                .map(|authorization_rules_program| authorization_rules_program.to_account_info()),
            authorization_rules: accounts
                .authorization_rules
                .as_ref()
                .map(|authorization_rules| authorization_rules.to_account_info()),
        };

        revoke_metadata_delegate(revoke_accounts)?;

        // approve a new metadata delegate

        let new_delegate_record = accounts
            .new_delegate_record
            .as_ref()
            .ok_or(CandyError::MissingMetadataDelegateRecord)?;

        let delegate_accounts = ApproveMetadataDelegateHelperAccounts {
            authority_pda: accounts.authority_pda.to_account_info(),
            collection_metadata: accounts.new_collection_metadata.to_account_info(),
            collection_mint: accounts.new_collection_mint.to_account_info(),
            collection_update_authority: accounts.new_collection_update_authority.to_account_info(),
            delegate_record: new_delegate_record.to_account_info(),
            payer: accounts.payer.to_account_info(),
            system_program: accounts.system_program.to_account_info(),
            sysvar_instructions: accounts.sysvar_instructions.to_account_info(),
            authorization_rules_program: accounts
                .authorization_rules_program
                .as_ref()
                .map(|authorization_rules_program| authorization_rules_program.to_account_info()),
            authorization_rules: accounts
                .authorization_rules
                .as_ref()
                .map(|authorization_rules| authorization_rules.to_account_info()),
        };

        approve_metadata_delegate(delegate_accounts)
    } else {
        // revoking the existing collection authority

        let collection_authority_record = accounts
            .collection_authority_record
            .as_ref()
            .ok_or(CandyError::MissingCollectionAuthorityRecord)?;

        let revoke_accounts = RevokeCollectionAuthorityHelperAccounts {
            authority_pda: accounts.authority_pda.to_account_info(),
            collection_authority_record: collection_authority_record.to_account_info(),
            collection_metadata: accounts.new_collection_metadata.to_account_info(),
            collection_mint: accounts.new_collection_mint.to_account_info(),
            token_metadata_program: accounts.token_metadata_program.to_account_info(),
        };

        revoke_collection_authority_helper(
            revoke_accounts,
            candy_machine.key(),
            *ctx.bumps.get("authority_pda").unwrap(),
        )?;

        // approving the new collection authority

        let new_collection_authority_record = accounts
            .new_collection_authority_record
            .as_ref()
            .ok_or(CandyError::MissingCollectionAuthorityRecord)?;

        let approve_collection_authority_helper_accounts =
            ApproveCollectionAuthorityHelperAccounts {
                payer: accounts.payer.to_account_info(),
                authority_pda: accounts.authority_pda.to_account_info(),
                collection_update_authority: accounts
                    .new_collection_update_authority
                    .to_account_info(),
                collection_mint: accounts.new_collection_mint.to_account_info(),
                collection_metadata: accounts.new_collection_metadata.to_account_info(),
                collection_authority_record: new_collection_authority_record.to_account_info(),
                token_metadata_program: accounts.token_metadata_program.to_account_info(),
                system_program: accounts.system_program.to_account_info(),
            };

        approve_collection_authority_helper(approve_collection_authority_helper_accounts)
    }
}

/// Set the collection PDA for the candy machine
#[derive(Accounts)]
pub struct SetCollectionV2<'info> {
    #[account(mut, has_one = authority)]
    candy_machine: Box<Account<'info, CandyMachine>>,

    // candy machine authority
    authority: Signer<'info>,

    /// CHECK: account checked in seeds constraint
    #[account(
        mut, seeds = [AUTHORITY_SEED.as_bytes(), candy_machine.to_account_info().key.as_ref()],
        bump
    )]
    authority_pda: UncheckedAccount<'info>,

    // payer of the transaction
    payer: Signer<'info>,

    /// CHECK: account checked in CPI
    collection_update_authority: UncheckedAccount<'info>,

    /// CHECK: account checked in CPI
    collection_mint: UncheckedAccount<'info>,

    /// CHECK: account checked in CPI
    collection_metadata: UncheckedAccount<'info>,

    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_authority_record: Option<UncheckedAccount<'info>>,

    /// CHECK: account checked in CPI
    #[account(mut)]
    delegate_record: Option<UncheckedAccount<'info>>,

    // update authority of the new collection NFT
    #[account(mut)]
    new_collection_update_authority: Signer<'info>,

    /// CHECK: account checked in CPI
    new_collection_metadata: UncheckedAccount<'info>,

    /// CHECK: account checked in CPI
    new_collection_mint: UncheckedAccount<'info>,

    /// CHECK: account checked in CPI
    new_collection_master_edition: UncheckedAccount<'info>,

    /// CHECK: account checked in CPI
    #[account(mut)]
    new_collection_authority_record: Option<UncheckedAccount<'info>>,

    /// CHECK: account checked in CPI
    #[account(mut)]
    new_delegate_record: Option<UncheckedAccount<'info>>,

    /// CHECK: account checked in CPI
    #[account(address = mpl_token_metadata::id())]
    token_metadata_program: UncheckedAccount<'info>,

    /// System program account.
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
