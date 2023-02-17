use anchor_lang::{prelude::*, solana_program::sysvar};
use mpl_token_metadata::{
    instruction::{
        builders::{DelegateBuilder, RevokeBuilder},
        revoke_collection_authority, DelegateArgs, InstructionBuilder, RevokeArgs,
    },
    state::{Metadata, TokenMetadataAccount, TokenStandard},
};
use solana_program::program::{invoke, invoke_signed};

use crate::{cmp_pubkeys, constants::AUTHORITY_SEED, CandyError, CandyMachine, PNFT_FEATURE};

use super::set_collection::{
    approve_collection_authority_helper, ApproveCollectionAuthorityHelperAccounts,
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
        // revoking the legacy collection authority

        let collection_authority_record = ctx
            .accounts
            .collection_authority_record
            .as_ref()
            .ok_or(CandyError::MissingCollectionAuthorityRecord)?;

        let revoke_collection_infos = vec![
            collection_authority_record.to_account_info(),
            ctx.accounts.authority_pda.to_account_info(),
            ctx.accounts.collection_metadata.to_account_info(),
            ctx.accounts.collection_mint.to_account_info(),
        ];

        let candy_machine_key = candy_machine.key();

        let authority_seeds = [
            AUTHORITY_SEED.as_bytes(),
            candy_machine_key.as_ref(),
            &[*ctx.bumps.get("authority_pda").unwrap()],
        ];

        invoke_signed(
            &revoke_collection_authority(
                ctx.accounts.token_metadata_program.key(),
                collection_authority_record.key(),
                ctx.accounts.authority_pda.key(),
                ctx.accounts.authority_pda.key(),
                ctx.accounts.collection_metadata.key(),
                ctx.accounts.collection_mint.key(),
            ),
            revoke_collection_infos.as_slice(),
            &[&authority_seeds],
        )?;

        // setting a new delegate

        let delegate_record = ctx
            .accounts
            .delegate_record
            .as_ref()
            .ok_or(CandyError::MissingMetadataDelegateRecord)?;

        let mut delegate_builder = DelegateBuilder::new();
        delegate_builder
            .delegate_record(delegate_record.key())
            .delegate(ctx.accounts.authority_pda.key())
            .mint(ctx.accounts.collection_mint.key())
            .metadata(ctx.accounts.collection_metadata.key())
            .payer(ctx.accounts.payer.key())
            .authority(ctx.accounts.collection_update_authority.key());

        let mut delegate_infos = vec![
            delegate_record.to_account_info(),
            ctx.accounts.authority_pda.to_account_info(),
            ctx.accounts.collection_metadata.to_account_info(),
            ctx.accounts.collection_mint.to_account_info(),
            ctx.accounts.collection_update_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.sysvar_instructions.to_account_info(),
        ];

        if let Some(authorization_rules_program) = &ctx.accounts.authorization_rules_program {
            delegate_builder.authorization_rules_program(authorization_rules_program.key());
            delegate_infos.push(authorization_rules_program.to_account_info());
        }

        if let Some(authorization_rules) = &ctx.accounts.authorization_rules {
            delegate_builder.authorization_rules(authorization_rules.key());
            delegate_infos.push(authorization_rules.to_account_info());
        }

        let delegate_ix = delegate_builder
            .build(DelegateArgs::CollectionV1 {
                authorization_data: None,
            })
            .map_err(|_| CandyError::InstructionBuilderFailed)?
            .instruction();

        invoke(&delegate_ix, &delegate_infos)?;

        // enables minting pNFTs
        candy_machine.enable_feature(PNFT_FEATURE);
    } else if token_standard == TokenStandard::NonFungible as u8
        && candy_machine.is_enabled(PNFT_FEATURE)
    {
        // revoking the delegate

        let delegate_record = ctx
            .accounts
            .delegate_record
            .as_ref()
            .ok_or(CandyError::MissingMetadataDelegateRecord)?;

        let mut revoke_builder = RevokeBuilder::new();
        revoke_builder
            .delegate_record(delegate_record.key())
            .delegate(ctx.accounts.authority_pda.key())
            .mint(ctx.accounts.collection_mint.key())
            .metadata(ctx.accounts.collection_metadata.key())
            .payer(ctx.accounts.payer.key())
            .authority(ctx.accounts.collection_update_authority.key());

        let mut revoke_infos = vec![
            delegate_record.to_account_info(),
            ctx.accounts.authority_pda.to_account_info(),
            ctx.accounts.collection_metadata.to_account_info(),
            ctx.accounts.collection_mint.to_account_info(),
            ctx.accounts.collection_update_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.sysvar_instructions.to_account_info(),
        ];

        if let Some(authorization_rules_program) = &ctx.accounts.authorization_rules_program {
            revoke_builder.authorization_rules_program(authorization_rules_program.key());
            revoke_infos.push(authorization_rules_program.to_account_info());
        }

        if let Some(authorization_rules) = &ctx.accounts.authorization_rules {
            revoke_builder.authorization_rules(authorization_rules.key());
            revoke_infos.push(authorization_rules.to_account_info());
        }

        let revoke_ix = revoke_builder
            .build(RevokeArgs::CollectionV1)
            .map_err(|_| CandyError::InstructionBuilderFailed)?
            .instruction();

        invoke(&revoke_ix, &revoke_infos)?;

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

        approve_collection_authority_helper(approve_accounts)?;

        // disables minting pNFTs
        candy_machine.disable_feature(PNFT_FEATURE);
    }

    Ok(())
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
    pub sysvar_instructions: UncheckedAccount<'info>,

    /// CHECK: account checked in CPI
    #[account(address = mpl_token_auth_rules::id())]
    authorization_rules_program: Option<UncheckedAccount<'info>>,

    /// CHECK: account constraints checked in account trait
    #[account(owner = mpl_token_auth_rules::id())]
    pub authorization_rules: Option<UncheckedAccount<'info>>,
}
