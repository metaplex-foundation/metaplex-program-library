use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::{assert_signer, token::TokenTransferParams};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};

use crate::{
    assertions::{assert_owned_by, metadata::assert_currently_holding},
    error::MetadataError,
    instruction::TransferArgs,
    pda::find_master_edition_account,
    state::{Metadata, TokenMetadataAccount, TokenStandard},
    utils::{freeze, thaw, validate},
};

#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct AuthorizationData {
    pub derived_key_seeds: Option<Vec<Vec<u8>>>,
    pub leaf_info: Option<LeafInfo>,
    pub name: String,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct LeafInfo {
    pub leaf: [u8; 32],
    pub proof: Vec<[u8; 32]>,
}

impl LeafInfo {
    pub fn into_native(self) -> mpl_token_auth_rules::LeafInfo {
        mpl_token_auth_rules::LeafInfo {
            leaf: self.leaf,
            proof: self.proof,
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
    if let Some(delegate) = metadata.delegate {
        if !currently_holding && owner_info.key != &delegate {
            return Err(MetadataError::InvalidOwner.into());
        }
    } else {
        if !currently_holding {
            return Err(MetadataError::InvalidOwner.into());
        }
    }

    if let Some(token_standard) = metadata.token_standard {
        match token_standard {
            TokenStandard::ProgrammableNonFungible => {
                let authorization_data = args.get_data();

                if authorization_rules_opt_info.is_none() || authorization_data.is_none() {
                    return Err(MetadataError::MissingAuthorizationRules.into());
                }

                if metadata.programmable_config.is_none() {
                    return Err(MetadataError::MissingProgrammableConfig.into());
                }

                if edition_opt_info.is_none() {
                    return Err(MetadataError::MissingEditionAccount.into());
                }
                let master_edition_info = edition_opt_info.unwrap();

                let auth_pda = authorization_rules_opt_info.unwrap();
                let auth_data = authorization_data.unwrap();
                let amount = args.get_amount();

                validate(
                    owner_info,
                    auth_pda,
                    destination_owner_info,
                    auth_data,
                    Some(amount),
                );

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
                msg!("amount: {}", amount);

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
    } else {
        return Err(MetadataError::CouldNotDetermineTokenStandard.into());
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
        let account_info_iter = &mut accounts.iter().peekable();

        match self {
            TransferArgs::V1 { .. } => {
                let owner_info = next_account_info(account_info_iter)?;
                let token_account_info = next_account_info(account_info_iter)?;
                let metadata_info = next_account_info(account_info_iter)?;
                let mint_info = next_account_info(account_info_iter)?;

                let (edition_pda, _) = find_master_edition_account(mint_info.key);
                let edition_opt_info = account_info_iter.next_if(|a| a.key == &edition_pda);

                let destination_owner_info = next_account_info(account_info_iter)?;
                let destination_token_account_info = next_account_info(account_info_iter)?;

                let spl_token_program_info = next_account_info(account_info_iter)?;
                let spl_associated_token_program_info = next_account_info(account_info_iter)?;

                let system_program_info = next_account_info(account_info_iter)?;
                let sysvar_instructions_info = next_account_info(account_info_iter)?;

                // If the next account is the mpl_token_auth_rules ID, then we consume it
                // and read the next account which will be the authorization rules account.
                let authorization_rules_opt_info = if account_info_iter
                    .next_if(|a| a.key == &mpl_token_auth_rules::ID)
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

    fn get_data(&self) -> Option<&AuthorizationData> {
        match self {
            TransferArgs::V1 {
                authorization_data, ..
            } => authorization_data.as_ref(),
        }
    }

    fn get_amount(&self) -> u64 {
        match self {
            TransferArgs::V1 { amount, .. } => *amount,
        }
    }
}
