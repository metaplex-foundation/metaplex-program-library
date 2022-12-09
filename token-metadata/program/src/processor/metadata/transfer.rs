use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_auth_rules::{instruction::validate, state::Operation, Payload};
use mpl_utils::token::TokenTransferParams;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    error::MetadataError,
    instruction::TransferArgs,
    pda::find_master_edition_account,
    state::{Metadata, TokenMetadataAccount, TokenStandard},
    utils::{freeze, thaw},
};

#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct AuthorizationData {
    pub payload: Payload,
    pub name: String,
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
    _program_id: &Pubkey,
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
        _spl_associated_token_program: _,
        _system_program: _,
        _sysvar_instructions: _,
        authorization_rules,
    } = args.get_accounts(accounts)?;

    // Deserialize metadata to determine its type
    let metadata_data = Metadata::from_account_info(metadata)?;

    if let Some(token_standard) = metadata_data.token_standard {
        match token_standard {
            TokenStandard::ProgrammableNonFungible => {
                let authorization_data = args.get_data();

                if authorization_rules.is_none() || authorization_data.is_none() {
                    return Err(MetadataError::MissingAuthorizationRules.into());
                }

                if edition.is_none() {
                    return Err(MetadataError::MissingEditionAccount.into());
                }
                let master_edition = edition.unwrap();

                let auth_pda = authorization_rules.unwrap();
                let auth_data = authorization_data.unwrap();

                msg!("destination key: {:?}", destination_owner.key);
                msg!(
                    "payload destination key: {:?}",
                    auth_data.payload.destination_key.unwrap()
                );
                msg!("destination key owner: {:?}", destination_owner.owner);

                let validate_ix = validate(
                    mpl_token_auth_rules::ID,
                    *owner.key,
                    *auth_pda.key,
                    auth_data.name.clone(),
                    Operation::Transfer,
                    auth_data.payload.clone(),
                    vec![],
                    vec![*destination_owner.key],
                );

                invoke_signed(
                    &validate_ix,
                    &[owner.clone(), auth_pda.clone(), destination_owner.clone()],
                    &[],
                )
                .unwrap();

                thaw(mint, token_account, master_edition, spl_token_program)?;

                let token_transfer_params: TokenTransferParams = TokenTransferParams {
                    mint: mint.clone(),
                    source: token_account.clone(),
                    destination: destination_token_account.clone(),
                    amount: auth_data.payload.amount.unwrap(),
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
        _spl_associated_token_program: &'a AccountInfo<'a>,
        _system_program: &'a AccountInfo<'a>,
        _sysvar_instructions: &'a AccountInfo<'a>,
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

                let _spl_associated_token_program = next_account_info(account_info_iter)?;
                let _system_program = next_account_info(account_info_iter)?;
                let _sysvar_instructions = next_account_info(account_info_iter)?;

                // let maybe_next_account = account_info_iter.peek();

                // let authorization_rules = if maybe_next_account.is_some()
                //     && maybe_next_account.unwrap().key == &mpl_token_auth_rules::ID
                // {
                //     let _ = next_account_info(account_info_iter)?;
                //     Some(next_account_info(account_info_iter)?)
                // } else {
                //     None
                // };

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
                    _spl_associated_token_program,
                    _system_program,
                    _sysvar_instructions,
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
