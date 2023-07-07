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
    instruction::{Context, MetadataDelegateRole, Revoke, RevokeArgs},
    pda::{find_metadata_delegate_record_account, find_token_record_account},
    state::{
        Metadata, MetadataDelegateRecord, Resizable, TokenDelegateRole, TokenMetadataAccount,
        TokenRecord, TokenStandard,
    },
    utils::{clear_close_authority, freeze, thaw, ClearCloseAuthorityParams},
};

/// Revoke a delegation of the token.
pub fn revoke<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: RevokeArgs,
) -> ProgramResult {
    let context = Revoke::to_context(accounts)?;

    // checks if it is a TokenDelegate creation
    let token_delegate = match &args {
        // Sale
        RevokeArgs::SaleV1 => Some(TokenDelegateRole::Sale),
        // Transfer
        RevokeArgs::TransferV1 => Some(TokenDelegateRole::Transfer),
        // Utility
        RevokeArgs::UtilityV1 => Some(TokenDelegateRole::Utility),
        // Staking
        RevokeArgs::StakingV1 => Some(TokenDelegateRole::Staking),
        // Standard
        RevokeArgs::StandardV1 => Some(TokenDelegateRole::Standard),
        // LockedTransfer
        RevokeArgs::LockedTransferV1 => Some(TokenDelegateRole::LockedTransfer),
        // Migration
        RevokeArgs::MigrationV1 => Some(TokenDelegateRole::Migration),
        // we don't need to fail if did not find a match at this point
        _ => None,
    };

    if let Some(role) = token_delegate {
        // proceed with the delegate revoke if we have a match
        return revoke_persistent_delegate_v1(program_id, context, role);
    }

    // checks if it is a MetadataDelegate creation
    let metadata_delegate = match &args {
        RevokeArgs::CollectionV1 => Some(MetadataDelegateRole::Collection),
        RevokeArgs::DataV1 => Some(MetadataDelegateRole::Data),
        RevokeArgs::ProgrammableConfigV1 => Some(MetadataDelegateRole::ProgrammableConfig),
        RevokeArgs::AuthorityItemV1 => Some(MetadataDelegateRole::AuthorityItem),
        RevokeArgs::DataItemV1 => Some(MetadataDelegateRole::DataItem),
        RevokeArgs::CollectionItemV1 => Some(MetadataDelegateRole::CollectionItem),
        RevokeArgs::ProgrammableConfigItemV1 => Some(MetadataDelegateRole::ProgrammableConfigItem),
        // we don't need to fail if did not find a match at this point
        _ => None,
    };

    if let Some(role) = metadata_delegate {
        return revoke_delegate_v1(program_id, context, role);
    }

    // this only happens if we did not find a match
    Err(MetadataError::InvalidDelegateArgs.into())
}

fn revoke_delegate_v1(
    program_id: &Pubkey,
    ctx: Context<Revoke>,
    role: MetadataDelegateRole,
) -> ProgramResult {
    // signers

    assert_signer(ctx.accounts.payer_info)?;
    assert_signer(ctx.accounts.authority_info)?;

    // ownership

    assert_owned_by(ctx.accounts.metadata_info, program_id)?;
    assert_owned_by(ctx.accounts.mint_info, &spl_token::ID)?;

    // key match

    assert_keys_equal(ctx.accounts.system_program_info.key, &system_program::ID)?;
    assert_keys_equal(
        ctx.accounts.sysvar_instructions_info.key,
        &sysvar::instructions::ID,
    )?;

    // account relationships

    let delegate_record_info = match ctx.accounts.delegate_record_info {
        Some(delegate_record_info) => delegate_record_info,
        None => {
            return Err(MetadataError::MissingDelegateRecord.into());
        }
    };

    let metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    // there are two scenarios here:
    //   1. authority is equal to delegate: delegate as a signer is self-revoking
    //   2. otherwise we need the update authority as a signer
    let approver = if cmp_pubkeys(
        ctx.accounts.delegate_info.key,
        ctx.accounts.authority_info.key,
    ) {
        match MetadataDelegateRecord::from_account_info(delegate_record_info) {
            Ok(delegate_record) => {
                if cmp_pubkeys(&delegate_record.delegate, ctx.accounts.authority_info.key) {
                    delegate_record.update_authority
                } else {
                    return Err(MetadataError::InvalidDelegate.into());
                }
            }
            Err(_) => {
                return Err(MetadataError::DelegateNotFound.into());
            }
        }
    } else {
        assert_update_authority_is_correct(&metadata, ctx.accounts.authority_info)?;
        *ctx.accounts.authority_info.key
    };

    if metadata.mint != *ctx.accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    // closes the delegate record

    close_delegate_record(
        role,
        delegate_record_info,
        ctx.accounts.delegate_info.key,
        ctx.accounts.mint_info.key,
        &approver,
        ctx.accounts.payer_info,
    )
}

fn revoke_persistent_delegate_v1(
    program_id: &Pubkey,
    ctx: Context<Revoke>,
    role: TokenDelegateRole,
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
    assert_signer(ctx.accounts.authority_info)?;

    // ownership

    assert_owned_by(ctx.accounts.metadata_info, program_id)?;
    assert_owned_by(ctx.accounts.mint_info, &spl_token::ID)?;
    assert_owned_by(token_info, &spl_token::ID)?;

    // key match

    assert_keys_equal(ctx.accounts.system_program_info.key, &system_program::ID)?;
    assert_keys_equal(
        ctx.accounts.sysvar_instructions_info.key,
        &sysvar::instructions::ID,
    )?;
    assert_keys_equal(spl_token_program_info.key, &spl_token::ID)?;

    // account relationships

    let metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    if metadata.mint != *ctx.accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    // authority must be the owner of the token account: spl-token required the
    // token owner to revoke a delegate
    let token = Account::unpack(&token_info.try_borrow_data()?).unwrap();
    if token.owner != *ctx.accounts.authority_info.key {
        return Err(MetadataError::IncorrectOwner.into());
    }

    if let COption::Some(existing) = &token.delegate {
        if !cmp_pubkeys(existing, ctx.accounts.delegate_info.key) {
            return Err(MetadataError::InvalidDelegate.into());
        }
    } else {
        return Err(MetadataError::DelegateNotFound.into());
    }

    // process the revoke

    // programmables assets can have delegates from any role apart from `Standard`
    match metadata.token_standard {
        Some(TokenStandard::ProgrammableNonFungible)
        | Some(TokenStandard::ProgrammableNonFungibleEdition) => {
            if matches!(role, TokenDelegateRole::Standard) {
                return Err(MetadataError::InvalidDelegateRole.into());
            }

            let (mut token_record, token_record_info) = match ctx.accounts.token_record_info {
                Some(token_record_info) => {
                    let (pda_key, _) =
                        find_token_record_account(ctx.accounts.mint_info.key, token_info.key);

                    assert_keys_equal(&pda_key, token_record_info.key)?;
                    assert_owned_by(token_record_info, &crate::ID)?;

                    (
                        TokenRecord::from_account_info(token_record_info)?,
                        token_record_info,
                    )
                }
                None => {
                    // token record is required for programmable assets
                    return Err(MetadataError::MissingTokenRecord.into());
                }
            };

            if let Some(delegate) = token_record.delegate {
                assert_keys_equal(&delegate, ctx.accounts.delegate_info.key)?;

                if token_record.delegate_role == Some(role)
                    || token_record.delegate_role == Some(TokenDelegateRole::Migration)
                {
                    // resets the token record (state, rule_set_revision and delegate info)
                    token_record.reset();
                    token_record.save(
                        token_record_info,
                        ctx.accounts.payer_info,
                        ctx.accounts.system_program_info,
                    )?;
                } else {
                    return Err(MetadataError::InvalidDelegate.into());
                }
            }

            if let Some(master_edition_info) = ctx.accounts.master_edition_info {
                assert_owned_by(master_edition_info, &crate::ID)?;
                // derivation is checked on the thaw function
                thaw(
                    ctx.accounts.mint_info.clone(),
                    token_info.clone(),
                    master_edition_info.clone(),
                    spl_token_program_info.clone(),
                )?;

                // Clear the close authority if it's a Utility Delegate.
                if matches!(role, TokenDelegateRole::Utility) {
                    clear_close_authority(ClearCloseAuthorityParams {
                        token_info,
                        mint_info: ctx.accounts.mint_info,
                        token,
                        master_edition_info,
                        authority_info: master_edition_info,
                        spl_token_program_info,
                    })?;
                }
            } else {
                return Err(MetadataError::MissingEditionAccount.into());
            }
        }
        _ => {
            if !matches!(role, TokenDelegateRole::Standard) {
                return Err(MetadataError::InvalidDelegateRole.into());
            }
        }
    }

    // revokes the spl-token delegate
    invoke(
        &spl_token::instruction::revoke(
            spl_token_program_info.key,
            token_info.key,
            ctx.accounts.authority_info.key,
            &[],
        )?,
        &[token_info.clone(), ctx.accounts.authority_info.clone()],
    )?;

    if matches!(
        metadata.token_standard,
        Some(TokenStandard::ProgrammableNonFungible)
            | Some(TokenStandard::ProgrammableNonFungibleEdition)
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

    Ok(())
}

/// Closes a delegate PDA.
///
/// It checks that the derivation is correct before closing
/// the delegate record account.
fn close_delegate_record<'a>(
    role: MetadataDelegateRole,
    delegate_record_info: &'a AccountInfo<'a>,
    delegate: &Pubkey,
    mint: &Pubkey,
    approver: &Pubkey,
    payer_info: &'a AccountInfo<'a>,
) -> ProgramResult {
    if delegate_record_info.data_is_empty() {
        return Err(MetadataError::Uninitialized.into());
    }

    let (pda_key, _) = find_metadata_delegate_record_account(mint, role, approver, delegate);

    if pda_key != *delegate_record_info.key {
        Err(MetadataError::DerivedKeyInvalid.into())
    } else {
        // closes the delegate account
        close_account_raw(payer_info, delegate_record_info)
    }
}
