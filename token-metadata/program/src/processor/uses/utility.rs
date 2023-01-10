use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke, pubkey::Pubkey,
    system_program, sysvar,
};
use spl_token::{
    instruction::{freeze_account, thaw_account},
    state::Mint,
};

use crate::{
    assertions::assert_keys_equal,
    error::MetadataError,
    instruction::{Context, DelegateRole, Utility, UtilityArgs},
    state::{
        AssetState, AuthorityRequest, AuthorityType, Metadata, TokenMetadataAccount, TokenStandard,
    },
    utils::{
        assert_delegated_tokens, assert_freeze_authority_matches_mint, assert_initialized,
        assert_owned_by, freeze, thaw,
    },
};

pub fn utility<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: UtilityArgs,
) -> ProgramResult {
    let context = Utility::to_context(accounts)?;

    match args {
        UtilityArgs::LockV1 { .. } => toggle_lock_v1(program_id, context, args, true),
        UtilityArgs::UnlockV1 { .. } => toggle_lock_v1(program_id, context, args, false),
    }
}

fn toggle_lock_v1(
    program_id: &Pubkey,
    ctx: Context<Utility>,
    _args: UtilityArgs,
    lock: bool,
) -> ProgramResult {
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

    let mut metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    // mint must match mint account key
    if metadata.mint != *ctx.accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    // approver authority – this can be either:
    //  1. token owner: approver == token.owner
    //  2. spl-delegate: for non-programmable assets, approver == token.delegate
    //  3. delegate: valid delegate_record

    let authority_type = AuthorityType::get_authority_type(AuthorityRequest {
        authority: ctx.accounts.approver_info.key,
        update_authority: &metadata.update_authority,
        mint: ctx.accounts.mint_info.key,
        token_info: ctx.accounts.token_info,
        delegate_record_info: ctx.accounts.delegate_record_info,
        delegate_role: Some(DelegateRole::Utility),
    })?;

    let has_authority = match authority_type {
        AuthorityType::Holder | AuthorityType::Delegate => true,
        _ => {
            // check if the approver has a spl-token delegate (we can only do this if
            // we have the token account)
            if let Some(token_info) = ctx.accounts.token_info {
                assert_delegated_tokens(
                    ctx.accounts.approver_info,
                    ctx.accounts.mint_info,
                    token_info,
                )?;
                true
            } else {
                false
            }
        }
    };

    if !has_authority {
        // approver does not have authority to lock/unlock
        return Err(MetadataError::InvalidAuthorityType.into());
    }

    if !matches!(
        metadata.token_standard,
        Some(TokenStandard::ProgrammableNonFungible)
    ) {
        // for non-programmable assets, we need to freeze the token account,
        // which requires the freeze_authority/master_edition, token and spl-token progran accounts
        // to be on the transaction

        let (freeze_authority, is_master_edition) = match ctx.accounts.edition_info {
            Some(master_edition_info) => {
                assert_owned_by(master_edition_info, &crate::ID)?;
                (master_edition_info, true)
            }
            None => (ctx.accounts.approver_info, false),
        };

        // make sure we got the freeze authority
        let mint: Mint = assert_initialized(ctx.accounts.mint_info)?;
        assert_freeze_authority_matches_mint(&mint.freeze_authority, freeze_authority)?;

        let token_info = match ctx.accounts.token_info {
            Some(token_info) => token_info,
            None => {
                return Err(MetadataError::MissingTokenAccount.into());
            }
        };

        let spl_token_program_info = match ctx.accounts.spl_token_program_info {
            Some(spl_token_program_info) => {
                assert_keys_equal(spl_token_program_info.key, &spl_token::ID)?;
                spl_token_program_info
            }
            None => {
                return Err(MetadataError::MissingSplTokenProgram.into());
            }
        };

        if lock {
            if is_master_edition {
                // this will validate the master_edition derivation
                freeze(
                    ctx.accounts.mint_info.clone(),
                    token_info.clone(),
                    freeze_authority.clone(),
                    spl_token_program_info.clone(),
                )?;
            } else {
                // for fungible assets, we invoke spl-token directly
                // since we have the freeze authority
                invoke(
                    &freeze_account(
                        spl_token_program_info.key,
                        token_info.key,
                        ctx.accounts.mint_info.key,
                        freeze_authority.key,
                        &[],
                    )?,
                    &[
                        token_info.clone(),
                        ctx.accounts.mint_info.clone(),
                        freeze_authority.clone(),
                    ],
                )?;
            }
        } else if is_master_edition {
            // this will validate the master_edition derivation
            thaw(
                ctx.accounts.mint_info.clone(),
                token_info.clone(),
                freeze_authority.clone(),
                spl_token_program_info.clone(),
            )?;
        } else {
            // for fungible assets, we invoke spl-token directly
            // since we have the freeze authority
            invoke(
                &thaw_account(
                    spl_token_program_info.key,
                    token_info.key,
                    ctx.accounts.mint_info.key,
                    freeze_authority.key,
                    &[],
                )?,
                &[
                    token_info.clone(),
                    ctx.accounts.mint_info.clone(),
                    freeze_authority.clone(),
                ],
            )?;
        }
    }

    metadata.asset_state = Some(if lock {
        AssetState::Locked
    } else {
        AssetState::Unlocked
    });

    // save the state
    metadata.save(&mut ctx.accounts.metadata_info.try_borrow_mut_data()?)?;

    Ok(())
}
