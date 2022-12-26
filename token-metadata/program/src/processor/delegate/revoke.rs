use mpl_utils::{assert_signer, close_account_raw, cmp_pubkeys};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke, program_option::COption,
    program_pack::Pack, pubkey::Pubkey,
};
use spl_token::state::Account;

use crate::{
    assertions::{
        assert_derivation, assert_owned_by, metadata::assert_update_authority_is_correct,
    },
    error::MetadataError,
    instruction::{Context, DelegateRole, Revoke, RevokeArgs},
    state::{Metadata, TokenMetadataAccount, TokenStandard},
    utils::{freeze, thaw},
};

/// Revoke a delegation of the token.
///
/// # Accounts:
///
///   0. `[writable]` Delegate account key
///   1. `[]` Delegated owner
///   2. `[]` Mint account
///   3. `[writable]` Metadata account
///   4. `[optional]` Master Edition account
///   5. `[signer]` Authority to approve the delegation
///   6. `[signer, writable]` Payer
///   7. `[]` System Program
///   8. `[]` Instructions sysvar account
///   9. `[optional]` SPL Token Program
///   10. `[optional, writable]` Token account
///   11. `[optional]` Token Authorization Rules program
///   12. `[optional]` Token Authorization Rules account
pub fn revoke<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: RevokeArgs,
) -> ProgramResult {
    let context = Revoke::to_context(accounts)?;

    match args {
        RevokeArgs::CollectionV1 => revoke_collection_v1(program_id, context, args),
        RevokeArgs::SaleV1 => {
            // the sale delegate is a special type of transfer
            revoke_transfer_v1(program_id, context, args, DelegateRole::Sale)
        }
        RevokeArgs::TransferV1 => {
            revoke_transfer_v1(program_id, context, args, DelegateRole::Transfer)
        }
    }
}

fn revoke_collection_v1(
    program_id: &Pubkey,
    ctx: Context<Revoke>,
    _args: RevokeArgs,
) -> ProgramResult {
    // validates accounts

    assert_owned_by(ctx.accounts.metadata_info, program_id)?;
    assert_owned_by(ctx.accounts.mint_info, &spl_token::id())?;

    let asset_metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    assert_update_authority_is_correct(&asset_metadata, ctx.accounts.authority_info)?;

    if asset_metadata.mint != *ctx.accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    assert_signer(ctx.accounts.payer_info)?;
    assert_signer(ctx.accounts.authority_info)?;

    if ctx.accounts.delegate_record_info.data_is_empty() {
        return Err(MetadataError::Uninitialized.into());
    }

    // process the delegation creation (the derivation is checked
    // by the create helper)

    revoke_delegate(
        program_id,
        DelegateRole::Collection,
        ctx.accounts.delegate_record_info,
        ctx.accounts.delegate_info,
        ctx.accounts.mint_info,
        ctx.accounts.authority_info,
        ctx.accounts.payer_info,
    )
}

fn revoke_transfer_v1(
    program_id: &Pubkey,
    ctx: Context<Revoke>,
    _args: RevokeArgs,
    role: DelegateRole,
) -> ProgramResult {
    // validates accounts

    assert_owned_by(ctx.accounts.metadata_info, program_id)?;
    assert_owned_by(ctx.accounts.mint_info, &spl_token::id())?;
    assert_signer(ctx.accounts.payer_info)?;

    // Transfer delegate must have a token account and spl token program
    if ctx.accounts.token_info.is_none() {
        return Err(MetadataError::MissingTokenAccount.into());
    }
    if ctx.accounts.spl_token_program_info.is_none() {
        return Err(MetadataError::MissingSplTokenProgram.into());
    }
    let token_info = ctx.accounts.token_info.unwrap();
    let spl_token_program_info = ctx.accounts.spl_token_program_info.unwrap();

    let mut asset_metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    if asset_metadata.mint != *ctx.accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    if let Some(existing) = &asset_metadata.delegate_state {
        if !cmp_pubkeys(&existing.delegate, ctx.accounts.delegate_info.key) || existing.role != role
        {
            return Err(MetadataError::InvalidDelegate.into());
        }
    } else {
        return Err(MetadataError::DelegateNotFound.into());
    }

    // authority must be the owner of the token account
    let token_account = Account::unpack(&token_info.try_borrow_data()?).unwrap();
    if token_account.owner != *ctx.accounts.authority_info.key {
        return Err(MetadataError::IncorrectOwner.into());
    }

    if let COption::Some(existing) = &token_account.delegate {
        if !cmp_pubkeys(existing, ctx.accounts.delegate_info.key) {
            return Err(MetadataError::InvalidDelegate.into());
        }
    } else {
        return Err(MetadataError::DelegateNotFound.into());
    }

    // and must be a signer of the transaction
    assert_signer(ctx.accounts.authority_info)?;

    // process the delegation

    if matches!(
        asset_metadata.token_standard,
        Some(TokenStandard::ProgrammableNonFungible)
    ) {
        if let Some(master_edition_info) = ctx.accounts.master_edition_info {
            thaw(
                ctx.accounts.mint_info.clone(),
                token_info.clone(),
                master_edition_info.clone(),
                spl_token_program_info.clone(),
            )?;
        } else {
            return Err(MetadataError::MissingEditionAccount.into());
        }
    }

    invoke(
        &spl_token::instruction::revoke(
            spl_token_program_info.key,
            token_info.key,
            ctx.accounts.authority_info.key,
            &[],
        )?,
        &[
            token_info.clone(),
            ctx.accounts.delegate_info.clone(),
            ctx.accounts.authority_info.clone(),
        ],
    )?;

    if matches!(
        asset_metadata.token_standard,
        Some(TokenStandard::ProgrammableNonFungible)
    ) {
        if let Some(master_edition_info) = ctx.accounts.master_edition_info {
            freeze(
                ctx.accounts.mint_info.clone(),
                token_info.clone(),
                master_edition_info.clone(),
                spl_token_program_info.clone(),
            )?;
        } else {
            return Err(MetadataError::MissingEditionAccount.into());
        }
    }

    // sale delegate is set to the metadata account
    asset_metadata.delegate_state = None;
    asset_metadata.save(&mut ctx.accounts.metadata_info.try_borrow_mut_data()?)?;

    Ok(())
}

fn revoke_delegate<'a>(
    program_id: &Pubkey,
    delegate_role: DelegateRole,
    delegate: &'a AccountInfo<'a>,
    delegate_owner: &'a AccountInfo<'a>,
    mint: &'a AccountInfo<'a>,
    owner: &'a AccountInfo<'a>,
    payer: &'a AccountInfo<'a>,
) -> ProgramResult {
    let role = delegate_role.to_string();
    // validates the delegate derivation
    let delegate_seeds = vec![
        mint.key.as_ref(),
        role.as_bytes(),
        delegate_owner.key.as_ref(),
        owner.key.as_ref(),
    ];
    assert_derivation(program_id, delegate, &delegate_seeds)?;
    // closes the delegate account
    close_account_raw(payer, delegate)
}
