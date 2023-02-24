use anchor_lang::{prelude::*, solana_program::sysvar};

use crate::{
    approve_metadata_delegate, cmp_pubkeys, constants::AUTHORITY_SEED,
    revoke_collection_authority_helper, revoke_metadata_delegate, AccountVersion,
    ApproveMetadataDelegateHelperAccounts, CandyError, CandyMachine,
    RevokeCollectionAuthorityHelperAccounts, RevokeMetadataDelegateHelperAccounts,
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

    if matches!(candy_machine.version, AccountVersion::V2) {
        // revoking the existing metadata delegate
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
            authorization_rules_program: None,
            authorization_rules: None,
        };

        revoke_metadata_delegate(revoke_accounts)?;
    } else {
        // revoking the existing collection authority
        let collection_authority_record = accounts
            .collection_authority_record
            .as_ref()
            .ok_or(CandyError::MissingCollectionAuthorityRecord)?;

        let revoke_accounts = RevokeCollectionAuthorityHelperAccounts {
            authority_pda: accounts.authority_pda.to_account_info(),
            collection_authority_record: collection_authority_record.to_account_info(),
            collection_metadata: accounts.collection_metadata.to_account_info(),
            collection_mint: accounts.collection_mint.to_account_info(),
            token_metadata_program: accounts.token_metadata_program.to_account_info(),
        };

        revoke_collection_authority_helper(
            revoke_accounts,
            candy_machine.key(),
            *ctx.bumps.get("authority_pda").unwrap(),
        )?;
        // bump the version of the account since we are setting a metadata delegate
        candy_machine.version = AccountVersion::V2;
    }

    // approve a new metadata delegate

    let delegate_accounts = ApproveMetadataDelegateHelperAccounts {
        authority_pda: accounts.authority_pda.to_account_info(),
        collection_metadata: accounts.new_collection_metadata.to_account_info(),
        collection_mint: accounts.new_collection_mint.to_account_info(),
        collection_update_authority: accounts.new_collection_update_authority.to_account_info(),
        delegate_record: accounts.new_delegate_record.to_account_info(),
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
}

/// Set the collection PDA for the candy machine.
///
/// # Accounts
///
///   0. `[writable]` Candy Machine account (must be pre-allocated but zero content)
///   1. `[signer]` Candy Machine authority
///   2. `[]` Authority PDA (seeds `["candy_machine", candy machine id]`)
///   3. `[signer]` Payer
///   4. `[]` Collection update authority
///   5. `[]` Collection mint
///   6. `[]` Collection metadata
///   7. `[optional, writable]` Metadata delegate record
///   8. `[optional, writable]` Collection authority record
///   9. `[signer]` New collection update authority
///   10. `[]` New collection mint
///   11. `[]` New collection metadata
///   12. `[]` New collection master edition
///   13. `[writable]` New metadata delegate record
///   14. `[]` Token Metadata program
///   15. `[]` System program
///   16. `[]` Instructions sysvar account
///   17. `[optional]` Token Authorization Rules program
///   18. `[optional]` Token authorization rules account
#[derive(Accounts)]
pub struct SetCollectionV2<'info> {
    /// Candy Machine account.
    #[account(mut, has_one = authority)]
    candy_machine: Box<Account<'info, CandyMachine>>,

    /// Candy Machine authority.
    authority: Signer<'info>,

    /// Authority PDA.
    ///
    /// CHECK: account checked in seeds constraint
    #[account(
        seeds = [AUTHORITY_SEED.as_bytes(), candy_machine.to_account_info().key.as_ref()],
        bump
    )]
    authority_pda: UncheckedAccount<'info>,

    /// Payer of the transaction.
    payer: Signer<'info>,

    /// Update authority of the collection.
    ///
    /// CHECK: account checked in CPI
    collection_update_authority: UncheckedAccount<'info>,

    /// Mint account of the collection.
    ///
    /// CHECK: account checked in CPI
    collection_mint: UncheckedAccount<'info>,

    /// Metadata account of the collection.
    ///
    /// CHECK: account checked in CPI
    collection_metadata: UncheckedAccount<'info>,

    /// Metadata delegate record.
    ///
    /// CHECK: account checked in CPI
    #[account(mut)]
    delegate_record: Option<UncheckedAccount<'info>>,

    /// Collection authority record.
    ///
    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_authority_record: Option<UncheckedAccount<'info>>,

    /// Update authority of the new collection NFT.
    new_collection_update_authority: Signer<'info>,

    /// New collection mint.
    ///
    /// CHECK: account checked in CPI
    new_collection_mint: UncheckedAccount<'info>,

    /// New collection metadata.
    ///
    /// CHECK: account checked in CPI
    new_collection_metadata: UncheckedAccount<'info>,

    /// New collection master edition.
    ///
    /// CHECK: account checked in CPI
    new_collection_master_edition: UncheckedAccount<'info>,

    /// New metadata delegate record.
    ///
    /// CHECK: account checked in CPI
    #[account(mut)]
    new_delegate_record: UncheckedAccount<'info>,

    /// Token Metadata program.
    ///
    /// CHECK: account checked in CPI
    #[account(address = mpl_token_metadata::id())]
    token_metadata_program: UncheckedAccount<'info>,

    /// System program.
    system_program: Program<'info, System>,

    /// Instructions sysvar account.
    ///
    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::instructions::id())]
    sysvar_instructions: UncheckedAccount<'info>,

    /// Token Authorization Rules program.
    ///
    /// CHECK: account checked in CPI
    #[account(address = mpl_token_auth_rules::id())]
    authorization_rules_program: Option<UncheckedAccount<'info>>,

    /// Token Authorization rules account for the collection metadata (if any).
    ///
    /// CHECK: account constraints checked in account trait
    #[account(owner = mpl_token_auth_rules::id())]
    authorization_rules: Option<UncheckedAccount<'info>>,
}
