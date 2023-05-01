use std::fmt::Display;

use mpl_token_auth_rules::processor::cmp_pubkeys;
use mpl_utils::{assert_signer, token::TokenTransferParams};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::invoke,
    program_error::ProgramError,
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    system_program,
    sysvar::{self, instructions::get_instruction_relative},
};
use spl_token::state::Account;

use crate::{
    assertions::{
        assert_keys_equal, assert_owned_by, assert_token_matches_owner_and_mint,
        metadata::assert_holding_amount,
    },
    error::MetadataError,
    instruction::{Context, Transfer, TransferArgs},
    pda::find_token_record_account,
    state::{
        AuthorityRequest, AuthorityResponse, AuthorityType, Metadata, Operation, TokenDelegateRole,
        TokenMetadataAccount, TokenRecord, TokenStandard,
    },
    utils::{
        assert_derivation, auth_rules_validate, clear_close_authority, close_program_account,
        create_token_record_account, frozen_transfer, AuthRulesValidateParams,
        ClearCloseAuthorityParams,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransferScenario {
    Holder,
    TransferDelegate,
    SaleDelegate,
    MigrationDelegate,
}

impl Display for TransferScenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Holder => write!(f, "Owner"),
            Self::TransferDelegate => write!(f, "TransferDelegate"),
            Self::SaleDelegate => write!(f, "SaleDelegate"),
            Self::MigrationDelegate => write!(f, "MigrationDelegate"),
        }
    }
}

impl From<TransferScenario> for TokenDelegateRole {
    fn from(delegate: TransferScenario) -> Self {
        match delegate {
            TransferScenario::TransferDelegate => TokenDelegateRole::Transfer,
            TransferScenario::SaleDelegate => TokenDelegateRole::Sale,
            TransferScenario::MigrationDelegate => TokenDelegateRole::Migration,
            _ => panic!("Invalid delegate role"),
        }
    }
}

impl From<TokenDelegateRole> for TransferScenario {
    fn from(delegate: TokenDelegateRole) -> Self {
        match delegate {
            TokenDelegateRole::Transfer => TransferScenario::TransferDelegate,
            TokenDelegateRole::Sale => TransferScenario::SaleDelegate,
            TokenDelegateRole::Migration => TransferScenario::MigrationDelegate,
            _ => panic!("Invalid delegate role"),
        }
    }
}

pub fn transfer<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: TransferArgs,
) -> ProgramResult {
    let context = Transfer::to_context(accounts)?;

    match args {
        TransferArgs::V1 { .. } => transfer_v1(program_id, context, args),
    }
}

fn transfer_v1(program_id: &Pubkey, ctx: Context<Transfer>, args: TransferArgs) -> ProgramResult {
    let TransferArgs::V1 {
        authorization_data: auth_data,
        amount,
    } = args;

    // Check signers

    // This authority must always be a signer, regardless of if it's the
    // actual token owner, a delegate or some other authority authorized
    // by a rule set.
    assert_signer(ctx.accounts.authority_info)?;

    // Assert program ownership.
    assert_owned_by(ctx.accounts.metadata_info, program_id)?;
    assert_owned_by(ctx.accounts.mint_info, &spl_token::ID)?;
    assert_owned_by(ctx.accounts.token_info, &spl_token::ID)?;
    if let Some(owner_token_record_info) = ctx.accounts.owner_token_record_info {
        assert_owned_by(owner_token_record_info, program_id)?;
    }
    if let Some(master_edition) = ctx.accounts.edition_info {
        assert_owned_by(master_edition, program_id)?;
    }
    if let Some(authorization_rules) = ctx.accounts.authorization_rules_info {
        assert_owned_by(authorization_rules, &mpl_token_auth_rules::ID)?;
    }

    // Check if the destination exists.
    if ctx.accounts.destination_info.data_is_empty() {
        // if the token account is empty, we will initialize a new one but it must
        // be a ATA account
        assert_derivation(
            &spl_associated_token_account::id(),
            ctx.accounts.destination_info,
            &[
                ctx.accounts.destination_owner_info.key.as_ref(),
                spl_token::id().as_ref(),
                ctx.accounts.mint_info.key.as_ref(),
            ],
        )?;

        // creating the associated token account
        invoke(
            &spl_associated_token_account::instruction::create_associated_token_account(
                ctx.accounts.payer_info.key,
                ctx.accounts.destination_owner_info.key,
                ctx.accounts.mint_info.key,
                &spl_token::id(),
            ),
            &[
                ctx.accounts.payer_info.clone(),
                ctx.accounts.destination_owner_info.clone(),
                ctx.accounts.mint_info.clone(),
                ctx.accounts.destination_info.clone(),
            ],
        )?;
    } else {
        assert_owned_by(ctx.accounts.destination_info, &spl_token::id())?;
        assert_token_matches_owner_and_mint(
            ctx.accounts.destination_info,
            ctx.accounts.destination_owner_info.key,
            ctx.accounts.mint_info.key,
        )?;
    }

    // Check program IDs.

    if ctx.accounts.spl_token_program_info.key != &spl_token::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if ctx.accounts.spl_ata_program_info.key != &spl_associated_token_account::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if ctx.accounts.system_program_info.key != &system_program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if ctx.accounts.sysvar_instructions_info.key != &sysvar::instructions::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if let Some(auth_rules_program) = ctx.accounts.authorization_rules_program_info {
        if auth_rules_program.key != &mpl_token_auth_rules::ID {
            return Err(ProgramError::IncorrectProgramId);
        }
    }

    let mut is_wallet_to_wallet = false;

    // Deserialize metadata.
    let metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;

    // Must be the actual current owner of the token where
    // mint, token, owner and metadata accounts all match up.
    assert_holding_amount(
        &crate::ID,
        ctx.accounts.token_owner_info,
        ctx.accounts.metadata_info,
        &metadata,
        ctx.accounts.mint_info,
        ctx.accounts.token_info,
        amount,
    )?;

    let token_transfer_params: TokenTransferParams = TokenTransferParams {
        mint: ctx.accounts.mint_info.clone(),
        source: ctx.accounts.token_info.clone(),
        destination: ctx.accounts.destination_info.clone(),
        amount,
        authority: ctx.accounts.authority_info.clone(),
        authority_signer_seeds: None,
        token_program: ctx.accounts.spl_token_program_info.clone(),
    };

    let token_standard = metadata.token_standard;
    let token = Account::unpack(&ctx.accounts.token_info.try_borrow_data()?)?;

    let AuthorityResponse { authority_type, .. } =
        AuthorityType::get_authority_type(AuthorityRequest {
            authority: ctx.accounts.authority_info.key,
            update_authority: &metadata.update_authority,
            mint: ctx.accounts.mint_info.key,
            token: Some(ctx.accounts.token_info.key),
            token_account: Some(&token),
            token_record_info: ctx.accounts.owner_token_record_info,
            token_delegate_roles: vec![
                TokenDelegateRole::Sale,
                TokenDelegateRole::Transfer,
                TokenDelegateRole::LockedTransfer,
                TokenDelegateRole::Migration,
            ],
            ..Default::default()
        })?;

    match authority_type {
        AuthorityType::Holder => {
            // Wallet-to-wallet are currently exempt from auth rules so we need to check this and pass it into
            // the auth rules validator function.
            //
            // This only applies to Holder transfers as we cannot prove a delegate transfer is
            // from a proper system wallet.

            // If the program id of the current instruction is anything other than our program id
            // we know this is a CPI call from another program.
            let current_ix =
                get_instruction_relative(0, ctx.accounts.sysvar_instructions_info).unwrap();

            let is_cpi = !cmp_pubkeys(&current_ix.program_id, &crate::ID);

            // This can be replaced with a sys call to curve25519 once that feature activates.
            let wallets_are_system_program_owned =
                cmp_pubkeys(ctx.accounts.token_owner_info.owner, &system_program::ID)
                    && cmp_pubkeys(
                        ctx.accounts.destination_owner_info.owner,
                        &system_program::ID,
                    );

            // The only case where a transfer is wallet-to-wallet is if the wallets are both owned by
            // the system program and it's not a CPI call. Holders have to be signers so we can reject
            // malicious PDA signers owned by the system program by rejecting CPI calls here.
            //
            // Legitimate programs can use initialized PDAs or multiple instructions with a temp program-owned
            // PDA to go around this restriction for cases where they are passing through a proper system wallet
            // signer via an invoke call.
            is_wallet_to_wallet = !is_cpi && wallets_are_system_program_owned;
        }
        AuthorityType::TokenDelegate => {
            // the delegate has already being validated, but we need to validate
            // that it can transfer the required amount
            if token.delegated_amount < amount || token.amount < amount {
                return Err(MetadataError::InsufficientTokenBalance.into());
            }
        }
        _ => {
            if matches!(token_standard, Some(TokenStandard::ProgrammableNonFungible)) {
                return Err(MetadataError::InvalidAuthorityType.into());
            }

            // the authority must be either the token owner or a delegate for the
            // transfer to succeed
            let available_amount = if cmp_pubkeys(&token.owner, ctx.accounts.authority_info.key) {
                token.amount
            } else if COption::from(*ctx.accounts.authority_info.key) == token.delegate {
                token.delegated_amount
            } else {
                return Err(MetadataError::InvalidAuthorityType.into());
            };

            if available_amount < amount {
                return Err(MetadataError::InsufficientTokenBalance.into());
            }
        }
    }

    match token_standard {
        Some(TokenStandard::ProgrammableNonFungible) => {
            // All pNFTs should have a token record passed in and existing.
            // The token delegate role may not be populated, however.
            let owner_token_record_info =
                if let Some(record_info) = ctx.accounts.owner_token_record_info {
                    record_info
                } else {
                    return Err(MetadataError::MissingTokenRecord.into());
                };

            let destination_token_record_info =
                if let Some(record_info) = ctx.accounts.destination_token_record_info {
                    record_info
                } else {
                    return Err(MetadataError::MissingTokenRecord.into());
                };

            let (pda_key, _) =
                find_token_record_account(ctx.accounts.mint_info.key, ctx.accounts.token_info.key);
            // validates the derivation
            assert_keys_equal(&pda_key, owner_token_record_info.key)?;

            let (new_pda_key, _) = find_token_record_account(
                ctx.accounts.mint_info.key,
                ctx.accounts.destination_info.key,
            );
            // validates the derivation
            assert_keys_equal(&new_pda_key, destination_token_record_info.key)?;

            let owner_token_record = TokenRecord::from_account_info(owner_token_record_info)?;

            let is_sale_delegate = owner_token_record
                .delegate_role
                .map(|role| role == TokenDelegateRole::Sale)
                .unwrap_or(false);

            let is_locked_transfer_delegate = owner_token_record
                .delegate_role
                .map(|role| role == TokenDelegateRole::LockedTransfer)
                .unwrap_or(false);

            let scenario = match authority_type {
                AuthorityType::Holder => {
                    if is_sale_delegate {
                        return Err(MetadataError::OnlySaleDelegateCanTransfer.into());
                    }
                    TransferScenario::Holder
                }
                AuthorityType::TokenDelegate => {
                    if owner_token_record.delegate_role.is_none() {
                        return Err(MetadataError::MissingDelegateRole.into());
                    }

                    // need to validate whether the destination key matches the locked
                    // transfer address
                    if is_locked_transfer_delegate {
                        let locked_address = owner_token_record
                            .locked_transfer
                            .ok_or(MetadataError::MissingLockedTransferAddress)?;

                        if !cmp_pubkeys(&locked_address, ctx.accounts.destination_owner_info.key) {
                            return Err(MetadataError::InvalidLockedTransferAddress.into());
                        }
                        // locked transfer is a special case of the transfer restricted to a specific
                        // address, so after validating the address we proceed as a 'normal' transfer
                        TokenDelegateRole::Transfer.into()
                    } else {
                        owner_token_record.delegate_role.unwrap().into()
                    }
                }
                _ => return Err(MetadataError::InvalidTransferAuthority.into()),
            };

            // Build our auth rules params.
            let auth_rules_validate_params = AuthRulesValidateParams {
                mint_info: ctx.accounts.mint_info,
                owner_info: None,
                authority_info: Some(ctx.accounts.authority_info),
                source_info: Some(ctx.accounts.token_owner_info),
                destination_info: Some(ctx.accounts.destination_owner_info),
                programmable_config: metadata.programmable_config,
                amount,
                auth_data,
                auth_rules_info: ctx.accounts.authorization_rules_info,
                operation: Operation::Transfer { scenario },
                is_wallet_to_wallet,
                rule_set_revision: owner_token_record
                    .rule_set_revision
                    .map(|revision| revision as usize),
            };

            auth_rules_validate(auth_rules_validate_params)?;
            frozen_transfer(token_transfer_params, ctx.accounts.edition_info)?;

            let master_edition_info = ctx
                .accounts
                .edition_info
                .ok_or(MetadataError::MissingEditionAccount)?;

            clear_close_authority(ClearCloseAuthorityParams {
                token_info: ctx.accounts.token_info,
                mint_info: ctx.accounts.mint_info,
                token,
                master_edition_info,
                authority_info: master_edition_info,
                spl_token_program_info: ctx.accounts.spl_token_program_info,
            })?;

            // If the token record account for the destination owner doesn't exist,
            // we create it.
            if destination_token_record_info.data_is_empty() {
                create_token_record_account(
                    program_id,
                    destination_token_record_info,
                    ctx.accounts.mint_info,
                    ctx.accounts.destination_info,
                    ctx.accounts.payer_info,
                    ctx.accounts.system_program_info,
                )?;
            }

            // Don't close token record if it's a self transfer.
            if owner_token_record_info.key != destination_token_record_info.key {
                // Close the source Token Record account, but do it after the CPI calls
                // so as to avoid Unbalanced Accounts errors due to the CPI context not knowing
                // about the manual lamport math done here.
                close_program_account(owner_token_record_info, ctx.accounts.payer_info)?;
            }
        }
        _ => mpl_utils::token::spl_token_transfer(token_transfer_params).unwrap(),
    }

    Ok(())
}
