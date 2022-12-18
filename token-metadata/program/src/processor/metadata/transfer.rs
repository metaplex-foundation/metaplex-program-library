use mpl_token_auth_rules::{
    payload::{PayloadKey, PayloadType},
    state::Operation,
};
use mpl_utils::{assert_signer, cmp_pubkeys, token::TokenTransferParams};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};

use crate::{
    assertions::{
        assert_owned_by, metadata::assert_currently_holding,
        programmable::assert_valid_authorization,
    },
    error::MetadataError,
    instruction::TransferArgs,
    pda::find_master_edition_account,
    processor::AuthorizationData,
    state::{Metadata, TokenMetadataAccount, TokenStandard},
    utils::{freeze, thaw, validate},
};

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
        edition_info_opt,
        destination_owner_info,
        destination_token_account_info,
        spl_token_program_info,
        spl_associated_token_program_info,
        system_program_info,
        sysvar_instructions_info,
        authorization_rules_info_opt,
    } = args.get_accounts(accounts)?;
    //** Account Validation **/
    msg!("Account Validation");

    // Check signers
    assert_signer(owner_info)?;
    // Additional account signers?

    // Assert program ownership
    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(mint_info, &spl_token::ID)?;

    if let Some(edition) = edition_info_opt {
        assert_owned_by(edition, program_id)?;
    }
    if let Some(authorization_rules) = authorization_rules_info_opt {
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
    if let Some(delegate) = metadata.delegate {
        if !currently_holding && owner_info.key != &delegate {
            return Err(MetadataError::InvalidOwner.into());
        }
    } else if !currently_holding {
        return Err(MetadataError::InvalidOwner.into());
    }

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
                assert_valid_authorization(
                    &authorization_data,
                    authorization_rules_info_opt,
                    config,
                )?;

                // We can safely unwrap here because they were all checked for existence
                // in the assertion above.
                let auth_pda = authorization_rules_info_opt.unwrap();
                let mut auth_data = authorization_data.unwrap();

                // Insert auth rules for Transfer
                auth_data
                    .payload
                    .insert(PayloadKey::Amount, PayloadType::Number(amount));
                auth_data.payload.insert(
                    PayloadKey::Target,
                    PayloadType::Pubkey(*destination_owner_info.key),
                );

                // This panics if the CPI into the auth rules program fails.
                validate(
                    owner_info,
                    auth_pda,
                    Operation::Transfer,
                    destination_owner_info,
                    &auth_data,
                )?;
            }

            // We need the edition account regardless of if there's a rule set,
            // because we need to thaw the token account.
            if edition_info_opt.is_none() {
                return Err(MetadataError::MissingEditionAccount.into());
            }
            let master_edition_info = edition_info_opt.unwrap();

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
        edition_info_opt: Option<&'a AccountInfo<'a>>,
        destination_owner_info: &'a AccountInfo<'a>,
        destination_token_account_info: &'a AccountInfo<'a>,
        spl_token_program_info: &'a AccountInfo<'a>,
        spl_associated_token_program_info: &'a AccountInfo<'a>,
        system_program_info: &'a AccountInfo<'a>,
        sysvar_instructions_info: &'a AccountInfo<'a>,
        authorization_rules_info_opt: Option<&'a AccountInfo<'a>>,
    },
}

impl TransferArgs {
    fn get_accounts<'a>(
        &self,
        accounts: &'a [AccountInfo<'a>],
    ) -> Result<TransferAccounts<'a>, ProgramError> {
        let account_info_iter = &mut accounts.iter().peekable();

        match self {
            TransferArgs::V1 { .. } => {
                let owner_info = next_account_info(account_info_iter)?;
                let token_account_info = next_account_info(account_info_iter)?;
                let metadata_info = next_account_info(account_info_iter)?;
                let mint_info = next_account_info(account_info_iter)?;

                let (edition_pda, _) = find_master_edition_account(mint_info.key);
                let edition_info_opt =
                    account_info_iter.next_if(|a| cmp_pubkeys(a.key, &edition_pda));

                let destination_owner_info = next_account_info(account_info_iter)?;
                let destination_token_account_info = next_account_info(account_info_iter)?;

                let spl_token_program_info = next_account_info(account_info_iter)?;
                let spl_associated_token_program_info = next_account_info(account_info_iter)?;

                let system_program_info = next_account_info(account_info_iter)?;
                let sysvar_instructions_info = next_account_info(account_info_iter)?;

                // If the next account is the mpl_token_auth_rules ID, then we consume it
                // and read the next account which will be the authorization rules account.
                let authorization_rules_info_opt = if account_info_iter
                    .next_if(|a| cmp_pubkeys(a.key, &mpl_token_auth_rules::ID))
                    .is_some()
                {
                    // Auth rules account
                    Some(next_account_info(account_info_iter)?)
                } else {
                    None
                };

                Ok(TransferAccounts::V1 {
                    owner_info,
                    token_account_info,
                    metadata_info,
                    mint_info,
                    edition_info_opt,
                    destination_owner_info,
                    destination_token_account_info,
                    spl_token_program_info,
                    spl_associated_token_program_info,
                    system_program_info,
                    sysvar_instructions_info,
                    authorization_rules_info_opt,
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
