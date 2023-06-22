mod lock;
mod unlock;

use borsh::BorshSerialize;
pub use lock::*;
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, system_program, sysvar,
};
use spl_token::{
    instruction::{freeze_account, thaw_account},
    state::{Account, Mint},
};
pub use unlock::*;

use crate::{
    assertions::{assert_keys_equal, metadata::assert_state},
    error::MetadataError,
    pda::find_token_record_account,
    state::{
        AuthorityRequest, AuthorityResponse, AuthorityType, Metadata, TokenDelegateRole,
        TokenMetadataAccount, TokenRecord, TokenStandard, TokenState,
    },
    utils::{
        assert_delegated_tokens, assert_freeze_authority_matches_mint, assert_initialized,
        assert_owned_by, freeze, thaw,
    },
};

pub(crate) struct ToggleAccounts<'a> {
    payer_info: &'a AccountInfo<'a>,
    authority_info: &'a AccountInfo<'a>,
    mint_info: &'a AccountInfo<'a>,
    token_info: &'a AccountInfo<'a>,
    metadata_info: &'a AccountInfo<'a>,
    edition_info: Option<&'a AccountInfo<'a>>,
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
    assert_owned_by(accounts.mint_info, &spl_token::ID)?;
    assert_owned_by(accounts.token_info, &spl_token::ID)?;

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

    let token = Account::unpack(&accounts.token_info.try_borrow_data()?)?;
    // token mint must match mint account key
    if token.mint != *accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }
    // and must have balance greater than 0 if we are locking
    if matches!(to, TokenState::Locked) && token.amount == 0 {
        return Err(MetadataError::InsufficientTokenBalance.into());
    }

    // authority – this can be either:
    //  1. token delegate (programmable non-fungible): valid token_record.delegate
    //  2. spl-delegate (non-fungibles): authority == token.delegate
    //  3. freeze authority (fungibles): authority == freeze_authority

    if matches!(
        metadata.token_standard,
        Some(TokenStandard::ProgrammableNonFungible)
    ) {
        let AuthorityResponse { authority_type, .. } =
            AuthorityType::get_authority_type(AuthorityRequest {
                precedence: &[AuthorityType::TokenDelegate],
                authority: accounts.authority_info.key,
                update_authority: &metadata.update_authority,
                mint: accounts.mint_info.key,
                token: Some(accounts.token_info.key),
                token_account: Some(&token),
                token_record_info: accounts.token_record_info,
                token_delegate_roles: vec![
                    TokenDelegateRole::Utility,
                    TokenDelegateRole::Staking,
                    TokenDelegateRole::LockedTransfer,
                    TokenDelegateRole::Migration,
                ],
                ..Default::default()
            })?;
        // only a delegate can lock/unlock
        if !matches!(authority_type, AuthorityType::TokenDelegate) {
            return Err(MetadataError::InvalidAuthorityType.into());
        }

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
        token_record
            .serialize(&mut *token_record_info.try_borrow_mut_data()?)
            .map_err(|_| MetadataError::BorshSerializationError.into())
    } else {
        let spl_token_program_info = match accounts.spl_token_program_info {
            Some(spl_token_program_info) => {
                assert_keys_equal(spl_token_program_info.key, &spl_token::ID)?;
                spl_token_program_info
            }
            None => {
                return Err(MetadataError::MissingSplTokenProgram.into());
            }
        };

        // we don't rely on the token standard to support legacy assets without
        // a token standard set; for non-fungibles, the (master) edition is the freeze
        // authority and we allow lock/unlock if the authority is a delegate; for
        // fungibles, the authority must match the freeze authority of the mint

        if let Some(edition_info) = accounts.edition_info {
            // check whether the authority is an spl-token delegate or not
            assert_delegated_tokens(
                accounts.authority_info,
                accounts.mint_info,
                accounts.token_info,
            )
            .map_err(|error| {
                let custom: ProgramError = MetadataError::InvalidDelegate.into();
                if error == custom {
                    MetadataError::InvalidAuthorityType.into()
                } else {
                    error
                }
            })?;

            match to {
                TokenState::Locked => {
                    // this will validate the (master) edition derivation, which
                    // is the freeze authority
                    freeze(
                        accounts.mint_info.clone(),
                        accounts.token_info.clone(),
                        edition_info.clone(),
                        spl_token_program_info.clone(),
                    )
                }
                TokenState::Unlocked => {
                    // this will validate the (master) edition derivation, which
                    // is the freeze authority
                    thaw(
                        accounts.mint_info.clone(),
                        accounts.token_info.clone(),
                        edition_info.clone(),
                        spl_token_program_info.clone(),
                    )
                }
                TokenState::Listed => Err(MetadataError::IncorrectTokenState.into()),
            }
        } else {
            // fungibles: the authority must be the mint freeze authority
            let mint: Mint = assert_initialized(accounts.mint_info)?;

            assert_freeze_authority_matches_mint(&mint.freeze_authority, accounts.authority_info)
                .map_err(|_| MetadataError::InvalidAuthorityType)?;

            match to {
                TokenState::Locked => {
                    // for fungible assets, we invoke spl-token directly
                    // since we have the freeze authority
                    invoke(
                        &freeze_account(
                            spl_token_program_info.key,
                            accounts.token_info.key,
                            accounts.mint_info.key,
                            accounts.authority_info.key,
                            &[],
                        )?,
                        &[
                            accounts.token_info.clone(),
                            accounts.mint_info.clone(),
                            accounts.authority_info.clone(),
                        ],
                    )
                }
                TokenState::Unlocked => {
                    // for fungible assets, we invoke spl-token directly
                    // since we have the freeze authority
                    invoke(
                        &thaw_account(
                            spl_token_program_info.key,
                            accounts.token_info.key,
                            accounts.mint_info.key,
                            accounts.authority_info.key,
                            &[],
                        )?,
                        &[
                            accounts.token_info.clone(),
                            accounts.mint_info.clone(),
                            accounts.authority_info.clone(),
                        ],
                    )
                }
                TokenState::Listed => Err(MetadataError::IncorrectTokenState.into()),
            }
        }
    }
}
