use mpl_utils::{assert_signer, token::TokenTransferParams};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, system_program, sysvar,
};

use crate::{
    assertions::{assert_delegate, assert_owned_by, metadata::assert_currently_holding},
    error::MetadataError,
    instruction::{DelegateRole, TransferArgs},
    processor::{try_get_account_info, try_get_optional_account_info, AuthorizationData},
    state::{Metadata, TokenMetadataAccount, TokenStandard},
    utils::{auth_rules_validate, frozen_transfer, AuthRulesValidateParams},
};

const EXPECTED_ACCOUNTS_LEN: usize = 15;

#[derive(Debug, PartialEq, Eq)]
enum TransferAuthority {
    Owner,
    TransferDelegate,
    SaleDelegate,
    AuthRules,
}

impl From<DelegateRole> for TransferAuthority {
    fn from(role: DelegateRole) -> Self {
        match role {
            DelegateRole::Transfer => Self::TransferDelegate,
            DelegateRole::Sale => Self::SaleDelegate,
            _ => panic!("Invalid delegate role"),
        }
    }
}

pub fn transfer<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: TransferArgs,
) -> ProgramResult {
    match args {
        TransferArgs::V1 { .. } => transfer_v1(program_id, accounts, args),
    }
}

fn transfer_v1<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: TransferArgs,
) -> ProgramResult {
    let TransferAccounts::V1 {
        owner_info,
        authority_info,
        token_account_info,
        metadata_info,
        mint_info,
        edition_opt_info,
        destination_owner_info,
        destination_token_account_info,
        delegate_record_opt_info,
        spl_token_program_info,
        spl_associated_token_program_info,
        system_program_info,
        sysvar_instructions_info,
        authorization_rules_opt_info,
    } = args.get_accounts(accounts)?;
    //** Account Validation **/
    msg!("Account Validation");

    // Check signers

    // This authority must always be a signer, regardless of if it's the
    // actual token owner, a delegate or some other authority authorized
    // by a rule set.
    assert_signer(authority_info)?;

    // Assert program ownership
    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(mint_info, &spl_token::ID)?;
    assert_owned_by(token_account_info, &spl_token::ID)?;

    if let Some(delegate_record_info) = delegate_record_opt_info {
        assert_owned_by(delegate_record_info, program_id)?;
    }
    if let Some(edition) = edition_opt_info {
        assert_owned_by(edition, program_id)?;
    }
    if let Some(authorization_rules) = authorization_rules_opt_info {
        assert_owned_by(authorization_rules, &mpl_token_auth_rules::ID)?;
    }

    // Check program IDs.
    msg!("Check program IDs");
    if spl_token_program_info.key != &spl_token::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if spl_associated_token_program_info.key != &spl_associated_token_account::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if system_program_info.key != &system_program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if sysvar_instructions_info.key != &sysvar::instructions::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Deserialize metadata.
    let metadata = Metadata::from_account_info(metadata_info)?;

    let amount = args.get_amount();
    let auth_data = args.get_auth_data();

    let token_transfer_params: TokenTransferParams = TokenTransferParams {
        mint: mint_info.clone(),
        source: token_account_info.clone(),
        destination: destination_token_account_info.clone(),
        amount,
        authority: authority_info.clone(),
        authority_signer_seeds: None,
        token_program: spl_token_program_info.clone(),
    };

    let auth_rules_validate_params = AuthRulesValidateParams {
        destination_owner_info,
        programmable_config: metadata.programmable_config.clone(),
        amount,
        auth_data,
        auth_rules_opt_info: authorization_rules_opt_info,
    };

    if metadata.token_standard.is_none() {
        return Err(MetadataError::InvalidTokenStandard.into());
    }
    let token_standard = metadata.token_standard.unwrap();

    // Sale delegates prevent any other kind of transfer.
    let is_sale_delegate_set = if let Some(ref delegate_state) = metadata.delegate_state {
        delegate_state.role == DelegateRole::Sale
    } else {
        false
    };

    // Wallet-to-wallet transfer are always allowed except if a sale
    // delegate is set.
    let is_wallet_to_wallet = owner_info.owner == &system_program::ID
        && destination_owner_info.owner == &system_program::ID;

    // Determine transfer type.
    let transfer_authority = if let Some(ref delegate_state) = metadata.delegate_state {
        // We have a delegate set on the metadata so we convert it
        // into our TransferAuthority enum.
        // This will panic for anything other than Transfer or Sale as
        // those are the only valid delegate roles that can be set on
        // the metadata.
        delegate_state.role.clone().into()
    } else if owner_info.key == authority_info.key {
        // Owner and authority are the same so this is a normal owner transfer.
        TransferAuthority::Owner
    } else {
        // The caller is using some other authority to transfer the token that
        // is not an owner or delegate.
        TransferAuthority::AuthRules
    };

    // Short circuit here if sale delegate is set and it's not a SaleDelegate trying
    // to transfer.
    if transfer_authority != TransferAuthority::SaleDelegate && is_sale_delegate_set {
        return Err(MetadataError::OnlySaleDelegateCanTransfer.into());
    }

    match transfer_authority {
        // Owner transfers are just a wrapper around SPL token transfer
        // for non-programmable assets.
        // Programmable assets have to follow auth rules.
        TransferAuthority::Owner => {
            msg!("Owner transfer");

            // Must be the actual current owner of the token where
            // mint, token, owner and metadata accounts all match up.
            assert_currently_holding(
                &crate::ID,
                owner_info,
                metadata_info,
                &metadata,
                mint_info,
                token_account_info,
            )?;

            // If it's not a wallet-to-wallet transfer and is a PNFT then
            // we follow authorization rules for the transfer.
            if !is_wallet_to_wallet
                && matches!(token_standard, TokenStandard::ProgrammableNonFungible)
            {
                msg!("Program transfer for PNFT");
                auth_rules_validate(auth_rules_validate_params)?;
                frozen_transfer(token_transfer_params, edition_opt_info)?;
                return Ok(());
            }
            // Otherwise, proceed to transfer normally.
        }
        // A SaleDelegate means no one except that delegate can transfer
        // and the transfer must follow auth rules.
        TransferAuthority::SaleDelegate => {
            // Validate the delegate
            assert_delegate(
                authority_info.key,
                DelegateRole::Sale,
                &metadata.delegate_state.unwrap(),
            )?;

            if matches!(token_standard, TokenStandard::ProgrammableNonFungible) {
                auth_rules_validate(auth_rules_validate_params)?;
                frozen_transfer(token_transfer_params, edition_opt_info)?;
                return Ok(());
            } else {
                panic!("Only programmable NFTs can have a sale delegate");
            }
        }
        // A TransferDelegate means PNFTs are subject auth rules,
        // but non-programmable NFTs are transferred normally by either
        // the owner or the delegate.
        TransferAuthority::TransferDelegate => {
            // Validate the delegate
            assert_delegate(
                authority_info.key,
                DelegateRole::Transfer,
                &metadata.delegate_state.unwrap(),
            )?;

            // If it's not a wallet-to-wallet transfer and is a PNFT then
            // we follow authorization rules for the transfer.
            if !is_wallet_to_wallet
                && matches!(token_standard, TokenStandard::ProgrammableNonFungible)
            {
                auth_rules_validate(auth_rules_validate_params)?;
                frozen_transfer(token_transfer_params, edition_opt_info)?;
                return Ok(());
            }
            // Otherwise, proceed to transfer normally.
        }
        // This is not an owner or a delegate but some other authority
        // allowed by auth rules, so must follow the rules and can only
        // be used for PNFTs.
        TransferAuthority::AuthRules => {
            if matches!(token_standard, TokenStandard::ProgrammableNonFungible) {
                auth_rules_validate(auth_rules_validate_params)?;
                frozen_transfer(token_transfer_params, edition_opt_info)?;
                return Ok(());
            } else {
                panic!("Only programmable NFTs can have a sale delegate");
            }
        }
    }

    match token_standard {
        TokenStandard::ProgrammableNonFungible => {
            msg!("Transferring PNFT");
            frozen_transfer(token_transfer_params, edition_opt_info)?
        }
        _ => {
            msg!("Transferring NFT normally");
            mpl_utils::token::spl_token_transfer(token_transfer_params).unwrap()
        }
    }

    Ok(())
}

enum TransferAccounts<'a> {
    V1 {
        owner_info: &'a AccountInfo<'a>,
        authority_info: &'a AccountInfo<'a>,
        token_account_info: &'a AccountInfo<'a>,
        metadata_info: &'a AccountInfo<'a>,
        mint_info: &'a AccountInfo<'a>,
        edition_opt_info: Option<&'a AccountInfo<'a>>,
        destination_owner_info: &'a AccountInfo<'a>,
        destination_token_account_info: &'a AccountInfo<'a>,
        delegate_record_opt_info: Option<&'a AccountInfo<'a>>,
        spl_token_program_info: &'a AccountInfo<'a>,
        spl_associated_token_program_info: &'a AccountInfo<'a>,
        system_program_info: &'a AccountInfo<'a>,
        sysvar_instructions_info: &'a AccountInfo<'a>,
        authorization_rules_opt_info: Option<&'a AccountInfo<'a>>,
    },
}

impl TransferArgs {
    fn get_accounts<'a>(
        &self,
        accounts: &'a [AccountInfo<'a>],
    ) -> Result<TransferAccounts<'a>, ProgramError> {
        // validates that we got the correct number of accounts
        if accounts.len() < EXPECTED_ACCOUNTS_LEN {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        match self {
            TransferArgs::V1 { .. } => {
                let owner_info = try_get_account_info(accounts, 0)?;
                let authority_info = try_get_account_info(accounts, 1)?;
                let token_account_info = try_get_account_info(accounts, 2)?;
                let metadata_info = try_get_account_info(accounts, 3)?;
                let mint_info = try_get_account_info(accounts, 4)?;
                let edition_opt_info = try_get_optional_account_info(accounts, 5)?;
                let destination_owner_info = try_get_account_info(accounts, 6)?;
                let destination_token_account_info = try_get_account_info(accounts, 7)?;
                let delegate_record_opt_info = try_get_optional_account_info(accounts, 8)?;
                let spl_token_program_info = try_get_account_info(accounts, 9)?;
                let spl_associated_token_program_info = try_get_account_info(accounts, 10)?;
                let system_program_info = try_get_account_info(accounts, 11)?;
                let sysvar_instructions_info = try_get_account_info(accounts, 12)?;
                let _mpl_token_auth_rules_info = try_get_optional_account_info(accounts, 13)?;
                let authorization_rules_opt_info = try_get_optional_account_info(accounts, 14)?;

                Ok(TransferAccounts::V1 {
                    owner_info,
                    authority_info,
                    token_account_info,
                    metadata_info,
                    mint_info,
                    edition_opt_info,
                    destination_owner_info,
                    destination_token_account_info,
                    delegate_record_opt_info,
                    spl_token_program_info,
                    spl_associated_token_program_info,
                    system_program_info,
                    sysvar_instructions_info,
                    authorization_rules_opt_info,
                })
            }
        }
    }

    pub(crate) fn get_auth_data(&self) -> Option<AuthorizationData> {
        match self {
            TransferArgs::V1 {
                authorization_data, ..
            } => authorization_data.clone(),
        }
    }

    pub(crate) fn get_amount(&self) -> u64 {
        match self {
            TransferArgs::V1 { amount, .. } => *amount,
        }
    }
}
