mod lock;
mod unlock;

pub use lock::*;

use borsh::BorshSerialize;
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke, pubkey::Pubkey,
    system_program, sysvar,
};
use spl_token::{
    instruction::{freeze_account, thaw_account},
    state::Mint,
};
pub use unlock::*;

use crate::{
    assertions::{assert_keys_equal, metadata::assert_state},
    error::MetadataError,
    pda::find_token_record_account,
    state::{
        AuthorityRequest, AuthorityType, Metadata, TokenDelegateRole, TokenMetadataAccount,
        TokenRecord, TokenStandard, TokenState,
    },
    utils::{
        assert_delegated_tokens, assert_freeze_authority_matches_mint, assert_initialized,
        assert_owned_by, freeze, thaw,
    },
};

pub(crate) struct ToggleAccounts<'a> {
    payer_info: &'a AccountInfo<'a>,
    approver_info: &'a AccountInfo<'a>,
    mint_info: &'a AccountInfo<'a>,
    token_info: Option<&'a AccountInfo<'a>>,
    metadata_info: &'a AccountInfo<'a>,
    master_edition_info: Option<&'a AccountInfo<'a>>,
    token_record_info: Option<&'a AccountInfo<'a>>,
    system_program_info: &'a AccountInfo<'a>,
    sysvar_instructions_info: &'a AccountInfo<'a>,
    spl_token_program_info: Option<&'a AccountInfo<'a>>,
}

pub(crate) fn toggle_asset_state(
    program_id: &Pubkey,
    accounts: ToggleAccounts,
    from: TokenState,
    to: TokenState,
) -> ProgramResult {
    // signers

    assert_signer(accounts.payer_info)?;
    assert_signer(accounts.approver_info)?;

    // ownership

    assert_owned_by(accounts.metadata_info, program_id)?;
    assert_owned_by(accounts.mint_info, &spl_token::id())?;

    // key match

    assert_keys_equal(accounts.system_program_info.key, &system_program::ID)?;
    assert_keys_equal(
        accounts.sysvar_instructions_info.key,
        &sysvar::instructions::ID,
    )?;

    // account relationships

    let metadata = Metadata::from_account_info(accounts.metadata_info)?;
    // mint must match mint account key
    if metadata.mint != *accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    // approver authority – this can be either:
    //  1. token owner: approver == token.owner
    //  2. spl-delegate: for non-programmable assets, approver == token.delegate
    //  3. token delegate: valid token_record.delegate

    let authority_type = AuthorityType::get_authority_type(AuthorityRequest {
        authority: accounts.approver_info.key,
        update_authority: &metadata.update_authority,
        mint: accounts.mint_info.key,
        token_info: accounts.token_info,
        metadata_delegate_record_info: None,
        metadata_delegate_role: None,
        token_record_info: accounts.token_record_info,
        token_delegate_roles: vec![TokenDelegateRole::Utility],
    })?;

    let has_authority = match authority_type {
        AuthorityType::Holder | AuthorityType::Delegate => true,
        _ => {
            if !matches!(
                metadata.token_standard,
                Some(TokenStandard::ProgrammableNonFungible),
            ) {
                // check if the approver has an spl-token delegate (we can only do this if
                // we have the token account)
                if let Some(token_info) = accounts.token_info {
                    assert_delegated_tokens(
                        accounts.approver_info,
                        accounts.mint_info,
                        token_info,
                    )?;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
    };

    if !has_authority {
        // approver does not have authority to lock/unlock
        return Err(MetadataError::InvalidAuthorityType.into());
    }

    if matches!(
        metadata.token_standard,
        Some(TokenStandard::ProgrammableNonFungible)
    ) {
        let (mut token_record, token_record_info) = match accounts.token_record_info {
            Some(token_record_info) => {
                let (pda_key, _) =
                    find_token_record_account(accounts.mint_info.key, accounts.approver_info.key);

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

        assert_state(&token_record, from)?;
        // for pNFTs, we only need to flip the programmable state
        token_record.state = to;
        // save the state
        token_record.serialize(&mut *token_record_info.try_borrow_mut_data()?)?;
    } else {
        // for non-programmable assets, we need to freeze the token account,
        // which requires the freeze_authority/master_edition, token and spl-token progran accounts
        // to be on the transaction

        let (freeze_authority, is_master_edition) = match accounts.master_edition_info {
            Some(master_edition_info) => {
                assert_owned_by(master_edition_info, &crate::ID)?;
                (master_edition_info, true)
            }
            None => (accounts.approver_info, false),
        };

        // make sure we got the freeze authority
        let mint: Mint = assert_initialized(accounts.mint_info)?;
        assert_freeze_authority_matches_mint(&mint.freeze_authority, freeze_authority)?;

        let token_info = match accounts.token_info {
            Some(token_info) => token_info,
            None => {
                return Err(MetadataError::MissingTokenAccount.into());
            }
        };

        let spl_token_program_info = match accounts.spl_token_program_info {
            Some(spl_token_program_info) => {
                assert_keys_equal(spl_token_program_info.key, &spl_token::ID)?;
                spl_token_program_info
            }
            None => {
                return Err(MetadataError::MissingSplTokenProgram.into());
            }
        };

        match to {
            TokenState::Locked => {
                if is_master_edition {
                    // this will validate the master_edition derivation
                    freeze(
                        accounts.mint_info.clone(),
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
                            accounts.mint_info.key,
                            freeze_authority.key,
                            &[],
                        )?,
                        &[
                            token_info.clone(),
                            accounts.mint_info.clone(),
                            freeze_authority.clone(),
                        ],
                    )?;
                }
            }
            TokenState::Unlocked => {
                if is_master_edition {
                    // this will validate the master_edition derivation
                    thaw(
                        accounts.mint_info.clone(),
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
                            accounts.mint_info.key,
                            freeze_authority.key,
                            &[],
                        )?,
                        &[
                            token_info.clone(),
                            accounts.mint_info.clone(),
                            freeze_authority.clone(),
                        ],
                    )?;
                }
            }
        }
    }

    Ok(())
}
