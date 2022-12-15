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
    assertions::assert_owned_by,
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
        token_account,
        metadata,
        mint,
        edition,
        owner,
        destination_token_account,
        destination_owner,
        spl_token_program,
        spl_associated_token_program,
        system_program,
        sysvar_instructions,
        authorization_rules,
    } = args.get_accounts(accounts)?;
    //** Account Validation **/
    // Check signers
    assert_signer(owner)?;
    // Additional account signers?

    // Assert program ownership
    assert_owned_by(metadata, program_id)?;
    assert_owned_by(token_account, &spl_token::id())?;
    assert_owned_by(mint, &spl_token::id())?;
    assert_owned_by(destination_token_account, &spl_token::id())?;

    if let Some(edition) = edition {
        assert_owned_by(edition, program_id)?;
    }
    if let Some(authorization_rules) = authorization_rules {
        assert_owned_by(authorization_rules, &mpl_token_auth_rules::ID)?;
    }

    // Check program IDs.
    if spl_token_program.key != &spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    if spl_associated_token_program.key != &spl_associated_token_account::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    if system_program.key != &solana_program::system_program::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    if sysvar_instructions.key != &sysvar::instructions::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Deserialize metadata to determine its type
    let metadata_data = Metadata::from_account_info(metadata)?;

    if let Some(token_standard) = metadata_data.token_standard {
        match token_standard {
            TokenStandard::ProgrammableNonFungible => {
                let authorization_data = args.get_data();

                if authorization_rules.is_none() || authorization_data.is_none() {
                    return Err(MetadataError::MissingAuthorizationRules.into());
                }

                if metadata_data.programmable_config.is_none() {
                    return Err(MetadataError::MissingProgrammableConfig.into());
                }

                if edition.is_none() {
                    return Err(MetadataError::MissingEditionAccount.into());
                }
                let master_edition = edition.unwrap();

                let auth_pda = authorization_rules.unwrap();
                let auth_data = authorization_data.unwrap();
                let amount = args.get_amount();

                validate(owner, auth_pda, destination_owner, auth_data, Some(amount));

                thaw(mint, token_account, master_edition, spl_token_program)?;

                let token_transfer_params: TokenTransferParams = TokenTransferParams {
                    mint: mint.clone(),
                    source: token_account.clone(),
                    destination: destination_token_account.clone(),
                    amount,
                    authority: owner.clone(),
                    authority_signer_seeds: None,
                    token_program: spl_token_program.clone(),
                };
                mpl_utils::token::spl_token_transfer(token_transfer_params).unwrap();

                freeze(mint, token_account, master_edition, spl_token_program)?;
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
                    mint: mint.clone(),
                    source: token_account.clone(),
                    destination: destination_token_account.clone(),
                    amount,
                    authority: owner.clone(),
                    authority_signer_seeds: None,
                    token_program: spl_token_program.clone(),
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
        token_account: &'a AccountInfo<'a>,
        metadata: &'a AccountInfo<'a>,
        mint: &'a AccountInfo<'a>,
        edition: Option<&'a AccountInfo<'a>>,
        owner: &'a AccountInfo<'a>,
        destination_token_account: &'a AccountInfo<'a>,
        destination_owner: &'a AccountInfo<'a>,
        spl_token_program: &'a AccountInfo<'a>,
        spl_associated_token_program: &'a AccountInfo<'a>,
        system_program: &'a AccountInfo<'a>,
        sysvar_instructions: &'a AccountInfo<'a>,
        authorization_rules: Option<&'a AccountInfo<'a>>,
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
                let token_account = next_account_info(account_info_iter)?;
                let metadata = next_account_info(account_info_iter)?;
                let mint = next_account_info(account_info_iter)?;

                let (edition_pda, _) = find_master_edition_account(mint.key);
                let edition = account_info_iter.next_if(|a| a.key == &edition_pda);

                let owner = next_account_info(account_info_iter)?;
                let destination_token_account = next_account_info(account_info_iter)?;
                let destination_owner = next_account_info(account_info_iter)?;
                let spl_token_program = next_account_info(account_info_iter)?;

                let spl_associated_token_program = next_account_info(account_info_iter)?;
                let system_program = next_account_info(account_info_iter)?;
                let sysvar_instructions = next_account_info(account_info_iter)?;

                // If the next account is the mpl_token_auth_rules ID, then we consume it
                // and read the next account which will be the authorization rules account.
                let authorization_rules = if account_info_iter
                    .next_if(|a| a.key == &mpl_token_auth_rules::ID)
                    .is_some()
                {
                    // Auth rules account
                    Some(next_account_info(account_info_iter)?)
                } else {
                    None
                };

                Ok(TransferAccounts::V1 {
                    token_account,
                    metadata,
                    mint,
                    edition,
                    owner,
                    destination_token_account,
                    destination_owner,
                    spl_token_program,
                    spl_associated_token_program,
                    system_program,
                    sysvar_instructions,
                    authorization_rules,
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
