use anchor_lang::{prelude::*, solana_program::sysvar};
use mpl_token_metadata::state::{Metadata, TokenMetadataAccount};

use crate::{
    approve_metadata_delegate, assert_token_standard, cmp_pubkeys, constants::AUTHORITY_SEED,
    revoke_collection_authority_helper, AccountVersion, ApproveMetadataDelegateHelperAccounts,
    CandyError, CandyMachine, RevokeCollectionAuthorityHelperAccounts,
};

pub fn set_token_standard(ctx: Context<SetTokenStandard>, token_standard: u8) -> Result<()> {
    let accounts = ctx.accounts;
    let candy_machine = &mut accounts.candy_machine;

    let collection_metadata_info = &accounts.collection_metadata;
    let collection_metadata: Metadata =
        Metadata::from_account_info(&collection_metadata_info.to_account_info())?;
    // check that the update authority matches the collection update authority
    if !cmp_pubkeys(&collection_metadata.mint, accounts.collection_mint.key) {
        return err!(CandyError::MintMismatch);
    }

    assert_token_standard(token_standard)?;

    if matches!(candy_machine.version, AccountVersion::V1) {
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

        // approve a new metadata delegate

        let delegate_accounts = ApproveMetadataDelegateHelperAccounts {
            authority_pda: accounts.authority_pda.to_account_info(),
            collection_metadata: accounts.collection_metadata.to_account_info(),
            collection_mint: accounts.collection_mint.to_account_info(),
            collection_update_authority: accounts.collection_update_authority.to_account_info(),
            delegate_record: accounts.collection_delegate_record.to_account_info(),
            payer: accounts.payer.to_account_info(),
            system_program: accounts.system_program.to_account_info(),
            sysvar_instructions: accounts.sysvar_instructions.to_account_info(),
            authorization_rules_program: accounts
                .authorization_rules_program
                .to_owned()
                .map(|authorization_rules_program| authorization_rules_program.to_account_info()),
            authorization_rules: accounts
                .authorization_rules
                .to_owned()
                .map(|authorization_rules| authorization_rules.to_account_info()),
        };

        approve_metadata_delegate(delegate_accounts)?;
        // bump the version of the account since we are setting a metadata delegate
        candy_machine.version = AccountVersion::V2;
    }

    msg!(
        "Changing token standard from {} to {}",
        candy_machine.token_standard,
        token_standard
    );

    candy_machine.token_standard = token_standard;

    Ok(())
}

/// Set the token standard to mint.
#[derive(Accounts)]
pub struct SetTokenStandard<'info> {
    /// Candy Machine account.
    #[account(mut, has_one = authority, has_one = collection_mint)]
    candy_machine: Account<'info, CandyMachine>,

    /// Candy Machine authority.
    authority: Signer<'info>,

    /// Authority PDA.
    ///
    /// CHECK: account checked in CPI
    #[account(
        mut,
        seeds = [AUTHORITY_SEED.as_bytes(), candy_machine.to_account_info().key.as_ref()],
        bump
    )]
    authority_pda: UncheckedAccount<'info>,

    /// Payer of the transaction.
    payer: Signer<'info>,

    /// Collection metadata delegate record.
    ///
    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_delegate_record: UncheckedAccount<'info>,

    /// Collection mint.
    ///
    /// CHECK: account checked in CPI
    collection_mint: UncheckedAccount<'info>,

    /// Collection metadata.
    ///
    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_metadata: UncheckedAccount<'info>,

    /// Collection authority record.
    ///
    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_authority_record: Option<UncheckedAccount<'info>>,

    /// Collection update authority.
    collection_update_authority: Signer<'info>,

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
