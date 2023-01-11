use mpl_utils::{assert_signer, close_account_raw, cmp_pubkeys};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke, program_option::COption,
    program_pack::Pack, pubkey::Pubkey, system_program, sysvar,
};
use spl_token::state::Account;

use crate::{
    assertions::{
        assert_keys_equal, assert_owned_by, metadata::assert_update_authority_is_correct,
    },
    error::MetadataError,
    instruction::{Context, DelegateRole, Revoke, RevokeArgs},
    pda::find_delegate_account,
    state::{DelegateRecord, Metadata, TokenMetadataAccount, TokenStandard},
    utils::{freeze, thaw},
};

/// Revoke a delegation of the token.
pub fn revoke<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: RevokeArgs,
) -> ProgramResult {
    let context = Revoke::to_context(accounts)?;

    match args {
        RevokeArgs::CollectionV1 => revoke_delegate(program_id, context, DelegateRole::Collection),
        RevokeArgs::SaleV1 => {
            // sale delegate is a special type of transfer
            revoke_persistent_delegate(program_id, context, DelegateRole::Sale)
        }
        RevokeArgs::TransferV1 => {
            revoke_persistent_delegate(program_id, context, DelegateRole::Transfer)
        }
        RevokeArgs::UpdateV1 => revoke_delegate(program_id, context, DelegateRole::Update),
        RevokeArgs::UtilityV1 => {
            revoke_persistent_delegate(program_id, context, DelegateRole::Utility)
        }
    }
}

fn revoke_delegate(program_id: &Pubkey, ctx: Context<Revoke>, role: DelegateRole) -> ProgramResult {
    // signers

    assert_signer(ctx.accounts.payer_info)?;
    assert_signer(ctx.accounts.approver_info)?;

    // ownership

    assert_owned_by(ctx.accounts.metadata_info, program_id)?;
    assert_owned_by(ctx.accounts.mint_info, &spl_token::id())?;

    // key match

    assert_keys_equal(ctx.accounts.system_program_info.key, &system_program::ID)?;
    assert_keys_equal(
        ctx.accounts.sysvar_instructions_info.key,
        &sysvar::instructions::ID,
    )?;

    // account relationships

    let metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    // there are two scenarios here:
    //   1. approver is equal to delegate: delegate as a signer is self-revoking
    //   2. otherwise we need the update authority as a signer
    let approver = if cmp_pubkeys(
        ctx.accounts.delegate_info.key,
        ctx.accounts.approver_info.key,
    ) {
        // retrieve the update authority since it was the pubkey that
        // approved the delegation
        metadata.update_authority
    } else {
        assert_update_authority_is_correct(&metadata, ctx.accounts.approver_info)?;
        *ctx.accounts.approver_info.key
    };

    if metadata.mint != *ctx.accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    // closes the delegate record

    close_delegate_record(
        role,
        ctx.accounts.delegate_record_info,
        ctx.accounts.delegate_info.key,
        ctx.accounts.mint_info.key,
        &approver,
        ctx.accounts.payer_info,
    )
}

fn revoke_persistent_delegate(
    program_id: &Pubkey,
    ctx: Context<Revoke>,
    role: DelegateRole,
) -> ProgramResult {
     // retrieving required optional accounts

     let token_info = match ctx.accounts.token_info {
        Some(token_info) => token_info,
        None => {
            return Err(MetadataError::MissingTokenAccount.into());
        }
    };

    let spl_token_program_info = match ctx.accounts.spl_token_program_info {
        Some(spl_token_program_info) => spl_token_program_info,
        None => {
            return Err(MetadataError::MissingSplTokenProgram.into());
        }
    };

    // signers

    assert_signer(ctx.accounts.payer_info)?;
    assert_signer(ctx.accounts.approver_info)?;

    // ownership

    assert_owned_by(ctx.accounts.metadata_info, program_id)?;
    assert_owned_by(ctx.accounts.mint_info, &spl_token::id())?;
    assert_owned_by(token_info, &spl_token::id())?;

    // key match

    assert_keys_equal(ctx.accounts.system_program_info.key, &system_program::ID)?;
    assert_keys_equal(ctx.accounts.sysvar_instructions_info.key, &sysvar::instructions::ID)?;
    assert_keys_equal(spl_token_program_info.key, &spl_token::ID)?;

    // account relationships

    let mut metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    if metadata.mint != *ctx.accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    // validates the persistent delegate information

    if let Some(delegate_role) = metadata.persistent_delegate {
        let delegate_record = DelegateRecord::from_account_info(ctx.accounts.delegate_record_info)?;

        if !(delegate_role == role
            && delegate_role == delegate_record.role
            && delegate_record.delegate == *ctx.accounts.delegate_info.key)
        {
            return Err(MetadataError::InvalidDelegate.into());
        }
    } else {
        return Err(MetadataError::DelegateNotFound.into());
    }

    // approver must be the owner of the token account
    let token_account = Account::unpack(&token_info.try_borrow_data()?).unwrap();
    if token_account.owner != *ctx.accounts.approver_info.key {
        return Err(MetadataError::IncorrectOwner.into());
    }

    if let COption::Some(existing) = &token_account.delegate {
        if !cmp_pubkeys(existing, ctx.accounts.delegate_info.key) {
            return Err(MetadataError::InvalidDelegate.into());
        }
    } else {
        return Err(MetadataError::DelegateNotFound.into());
    }

    // process revoke

    if matches!(
        metadata.token_standard,
        Some(TokenStandard::ProgrammableNonFungible)
    ) {
        if let Some(master_edition_info) = ctx.accounts.master_edition_info {
            assert_owned_by(master_edition_info, &crate::ID)?;
            // derivation is checked on the thaw function
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
            ctx.accounts.approver_info.key,
            &[],
        )?,
        &[
            token_info.clone(),
            ctx.accounts.delegate_info.clone(),
            ctx.accounts.approver_info.clone(),
        ],
    )?;

    if matches!(
        metadata.token_standard,
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
            // sanity check: this should not happen at this point since the master
            // edition account is validated before the delegation
            return Err(MetadataError::MissingEditionAccount.into());
        }
    }

    close_delegate_record(
        role,
        ctx.accounts.delegate_record_info,
        ctx.accounts.delegate_info.key,
        ctx.accounts.mint_info.key,
        ctx.accounts.approver_info.key,
        ctx.accounts.payer_info,
    )?;

    // clear the delegate information from metadata
    metadata.persistent_delegate = None;
    metadata.save(&mut ctx.accounts.metadata_info.try_borrow_mut_data()?)?;

    Ok(())
}

/// Closes a delegate PDA.
///
/// It checks that the derivation is correct before closing
/// the delegate record account.
fn close_delegate_record<'a>(
    role: DelegateRole,
    delegate_record_info: &'a AccountInfo<'a>,
    delegate: &Pubkey,
    mint: &Pubkey,
    approver: &Pubkey,
    payer_info: &'a AccountInfo<'a>,
) -> ProgramResult {
    if delegate_record_info.data_is_empty() {
        return Err(MetadataError::Uninitialized.into());
    }

    let (pda_key, _) = find_delegate_account(mint, role, approver, delegate);

    if pda_key != *delegate_record_info.key {
        Err(MetadataError::DerivedKeyInvalid.into())
    } else {
        // closes the delegate account
        close_account_raw(payer_info, delegate_record_info)
    }
}