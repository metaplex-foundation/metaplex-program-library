use mpl_token_auth_rules::payload::{PayloadKey, PayloadType};
use mpl_utils::{assert_signer, token::TokenTransferParams};
use num_traits::ToPrimitive;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, sysvar,
};

use crate::{
    assertions::{
        assert_owned_by, metadata::assert_currently_holding,
        programmable::assert_valid_authorization,
    },
    error::MetadataError,
    instruction::TransferArgs,
    processor::{try_get_account_info, try_get_optional_account_info, AuthorizationData},
    state::{Metadata, Operation, TokenMetadataAccount, TokenStandard},
    utils::{freeze, thaw, validate},
};

const EXPECTED_ACCOUNTS_LEN: usize = 13;

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
        token_account_info,
        metadata_info,
        mint_info,
        edition_opt_info,
        destination_owner_info,
        destination_token_account_info,
        spl_token_program_info,
        spl_associated_token_program_info,
        system_program_info,
        sysvar_instructions_info,
        authorization_rules_opt_info,
    } = args.get_accounts(accounts)?;
    //** Account Validation **/
    msg!("Account Validation");

    // Check signers
    assert_signer(owner_info)?;
    // Additional account signers?

    // Assert program ownership
    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(mint_info, &spl_token::ID)?;

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

    if system_program_info.key != &solana_program::system_program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if sysvar_instructions_info.key != &sysvar::instructions::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Deserialize metadata to determine its type
    let metadata = Metadata::from_account_info(metadata_info)?;

    // Check that owner account info is either the owner or the delegate.
    let currently_holding = assert_currently_holding(
        program_id,
        owner_info,
        metadata_info,
        &metadata,
        mint_info,
        token_account_info,
    )
    .is_ok();

    msg!("Must be Owner or Delegate.");
    // Use this for the payload operation in refactor.
    let operation = if let Some(delegate) = metadata.delegate {
        if !currently_holding && owner_info.key != &delegate {
            return Err(MetadataError::InvalidOwner.into());
        }
        Operation::Sale
    } else {
        if !currently_holding {
            return Err(MetadataError::InvalidOwner.into());
        }
        Operation::Transfer
    }
    .to_u16()
    .ok_or(MetadataError::InvalidOperation)?;

    if metadata.token_standard.is_none() {
        return Err(MetadataError::InvalidTokenStandard.into());
    }
    let token_standard = metadata.token_standard.unwrap();

    match token_standard {
        TokenStandard::ProgrammableNonFungible => {
            let amount = args.get_amount();

            msg!("checking programmable config");
            if let Some(ref config) = metadata.programmable_config {
                let authorization_data = args.get_auth_data();

                msg!("asserting valid authorization");
                assert_valid_authorization(authorization_rules_opt_info, config)?;

                // We can safely unwrap here because they were all checked for existence
                // in the assertion above.
                let auth_pda = authorization_rules_opt_info.unwrap();
                let mut auth_data = authorization_data.unwrap();

                // Insert auth rules for Transfer
                auth_data
                    .payload
                    .insert(PayloadKey::Amount, PayloadType::Number(amount));
                auth_data.payload.insert(
                    PayloadKey::Target,
                    PayloadType::Pubkey(*destination_owner_info.key),
                );

                // This panics if the CPI into the auth rules program fails, so unwrapping is ok.
                validate(auth_pda, operation, destination_owner_info, &auth_data)?;
            }

            // We need the edition account regardless of if there's a rule set,
            // because we need to thaw the token account.
            if edition_opt_info.is_none() {
                return Err(MetadataError::MissingEditionAccount.into());
            }
            let master_edition_info = edition_opt_info.unwrap();

            thaw(
                mint_info,
                token_account_info,
                master_edition_info,
                spl_token_program_info,
            )?;

            let token_transfer_params: TokenTransferParams = TokenTransferParams {
                mint: mint_info.clone(),
                source: token_account_info.clone(),
                destination: destination_token_account_info.clone(),
                amount,
                authority: owner_info.clone(),
                authority_signer_seeds: None,
                token_program: spl_token_program_info.clone(),
            };
            mpl_utils::token::spl_token_transfer(token_transfer_params).unwrap();

            freeze(
                mint_info,
                token_account_info,
                master_edition_info,
                spl_token_program_info,
            )?;
        }
        TokenStandard::NonFungible
        | TokenStandard::NonFungibleEdition
        | TokenStandard::Fungible
        | TokenStandard::FungibleAsset => {
            let amount = match token_standard {
                TokenStandard::NonFungible | TokenStandard::NonFungibleEdition => 1,
                TokenStandard::Fungible | TokenStandard::FungibleAsset => args.get_amount(),
                _ => panic!("Invalid token standard"),
            };

            msg!("Transferring NFT normally");
            let token_transfer_params: TokenTransferParams = TokenTransferParams {
                mint: mint_info.clone(),
                source: token_account_info.clone(),
                destination: destination_token_account_info.clone(),
                amount,
                authority: owner_info.clone(),
                authority_signer_seeds: None,
                token_program: spl_token_program_info.clone(),
            };
            mpl_utils::token::spl_token_transfer(token_transfer_params).unwrap();
        }
    }

    Ok(())
}

enum TransferAccounts<'a> {
    V1 {
        owner_info: &'a AccountInfo<'a>,
        token_account_info: &'a AccountInfo<'a>,
        metadata_info: &'a AccountInfo<'a>,
        mint_info: &'a AccountInfo<'a>,
        edition_opt_info: Option<&'a AccountInfo<'a>>,
        destination_owner_info: &'a AccountInfo<'a>,
        destination_token_account_info: &'a AccountInfo<'a>,
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
                let token_account_info = try_get_account_info(accounts, 1)?;
                let metadata_info = try_get_account_info(accounts, 2)?;
                let mint_info = try_get_account_info(accounts, 3)?;
                let edition_opt_info = try_get_optional_account_info(accounts, 4)?;
                let destination_owner_info = try_get_account_info(accounts, 5)?;
                let destination_token_account_info = try_get_account_info(accounts, 6)?;
                let spl_token_program_info = try_get_account_info(accounts, 7)?;
                let spl_associated_token_program_info = try_get_account_info(accounts, 8)?;
                let system_program_info = try_get_account_info(accounts, 9)?;
                let sysvar_instructions_info = try_get_account_info(accounts, 10)?;
                let _mpl_token_auth_rules_info = try_get_optional_account_info(accounts, 11)?;
                let authorization_rules_opt_info = try_get_optional_account_info(accounts, 12)?;

                Ok(TransferAccounts::V1 {
                    owner_info,
                    token_account_info,
                    metadata_info,
                    mint_info,
                    edition_opt_info,
                    destination_owner_info,
                    destination_token_account_info,
                    spl_token_program_info,
                    spl_associated_token_program_info,
                    system_program_info,
                    sysvar_instructions_info,
                    authorization_rules_opt_info,
                })
            }
        }
    }

    fn get_auth_data(&self) -> Option<AuthorizationData> {
        match self {
            TransferArgs::V1 {
                authorization_data, ..
            } => authorization_data.clone(),
        }
    }

    fn get_amount(&self) -> u64 {
        match self {
            TransferArgs::V1 { amount, .. } => *amount,
        }
    }
}
