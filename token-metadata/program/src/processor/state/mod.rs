mod lock;
mod unlock;

pub use lock::*;
pub use unlock::*;

use borsh::BorshSerialize;
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke, program_pack::Pack,
    pubkey::Pubkey, system_program, sysvar,
};
use spl_token::{
    instruction::{freeze_account, thaw_account},
    state::{Account, Mint},
};

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
    authority_info: &'a AccountInfo<'a>,
    token_owner_info: Option<&'a AccountInfo<'a>>,
    mint_info: &'a AccountInfo<'a>,
    token_info: &'a AccountInfo<'a>,
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
    assert_signer(accounts.authority_info)?;

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

    let token_account = Account::unpack(&accounts.token_info.try_borrow_data()?)?;
    // mint must match mint account key
    if token_account.mint != *accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    // approver authority – this can be either:
    //  1. token delegate: valid token_record.delegate
    //  2. spl-delegate: for non-programmable assets, approver == token.delegate

    let authority_type = AuthorityType::get_authority_type(AuthorityRequest {
        authority: accounts.authority_info.key,
        update_authority: &metadata.update_authority,
        mint: accounts.mint_info.key,
        token: Some(accounts.token_info.key),
        token_account: Some(&token_account),
        token_record_info: accounts.token_record_info,
        token_delegate_roles: vec![
            TokenDelegateRole::Utility,
            TokenDelegateRole::Staking,
            TokenDelegateRole::Migration,
        ],
        ..Default::default()
    })?;

    let has_authority = match authority_type {
        // holder is not allowed to lock/unlock
        AuthorityType::Holder => false,
        // (token) delegates can lock/unlock
        AuthorityType::Delegate => true,
        // if there is no authority, we checked if there is an spl-token
        // delegate set (this will be the case for non-programmable assets)
        _ => {
            if !matches!(
                metadata.token_standard,
                Some(TokenStandard::ProgrammableNonFungible),
            ) {
                // check if the approver has an spl-token delegate
                assert_delegated_tokens(
                    accounts.authority_info,
                    accounts.mint_info,
                    accounts.token_info,
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

    if matches!(
        metadata.token_standard,
        Some(TokenStandard::ProgrammableNonFungible)
    ) {
        let (mut token_record, token_record_info) = match accounts.token_record_info {
            Some(token_record_info) => {
                let (pda_key, _) =
                    find_token_record_account(accounts.mint_info.key, accounts.token_info.key);

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

        // make sure we are on the expected state
        assert_state(&token_record, from)?;
        // for pNFTs, we only need to flip the programmable state
        token_record.state = to;
        // save the state
        token_record.serialize(&mut *token_record_info.try_borrow_mut_data()?)?;
    } else {
        // for non-programmable assets, we need to freeze the token account,
        // which requires the freeze_authority/master_edition, token and spl-token program
        // accounts to be on the transaction

        let mint: Mint = assert_initialized(accounts.mint_info)?;

        let (freeze_authority, is_master_edition) = match accounts.master_edition_info {
            Some(master_edition_info) => {
                assert_owned_by(master_edition_info, &crate::ID)?;
                assert_freeze_authority_matches_mint(&mint.freeze_authority, master_edition_info)?;
                (master_edition_info, true)
            }
            None => {
                // in this case, the approver must be a spl-token delegate (which
                // has been already validated), so we need to validate that we have
                // the token owner

                let token_owner_info = match accounts.token_owner_info {
                    Some(token_owner_info) => token_owner_info,
                    None => {
                        return Err(MetadataError::MissingTokenOwnerAccount.into());
                    }
                };

                assert_keys_equal(token_owner_info.key, &token_account.owner)?;

                (token_owner_info, false)
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
                        accounts.token_info.clone(),
                        freeze_authority.clone(),
                        spl_token_program_info.clone(),
                    )?;
                } else {
                    // for fungible assets, we invoke spl-token directly
                    // since we have the freeze authority
                    invoke(
                        &freeze_account(
                            spl_token_program_info.key,
                            accounts.token_info.key,
                            accounts.mint_info.key,
                            freeze_authority.key,
                            &[],
                        )?,
                        &[
                            accounts.token_info.clone(),
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
                        accounts.token_info.clone(),
                        freeze_authority.clone(),
                        spl_token_program_info.clone(),
                    )?;
                } else {
                    // for fungible assets, we invoke spl-token directly
                    // since we have the freeze authority
                    invoke(
                        &thaw_account(
                            spl_token_program_info.key,
                            accounts.token_info.key,
                            accounts.mint_info.key,
                            freeze_authority.key,
                            &[],
                        )?,
                        &[
                            accounts.token_info.clone(),
                            accounts.mint_info.clone(),
                            freeze_authority.clone(),
                        ],
                    )?;
                }
            }
            TokenState::Listed => {
                return Err(MetadataError::IncorrectTokenState.into());
            }
        }
    }

    Ok(())
}
