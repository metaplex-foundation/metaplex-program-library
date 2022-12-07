use mpl_utils::{assert_initialized, cmp_pubkeys};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_token::{native_mint::DECIMALS, state::Mint};

use crate::{
    error::MetadataError,
    instruction::CreateMetadataArgs,
    state::{Metadata, TokenMetadataAccount, TokenStandard},
    utils::{
        create_master_edition, process_create_metadata_accounts_logic,
        CreateMetadataAccountsLogicArgs,
    },
};

/// Mint a new asset and associated metadata accounts.
///
/// # Accounts:
///
///   0. `[writable]` Metadata account
///   1. `[]` Mint account (signer when account is empty)
///   2. `[signer]` Mint authority
///   3. `[signer]` Payer
///   4. `[signer]` Update authority
///   5. `[]` System program
///   6. `[]` Instructions sysvar account
///   7. `[]` SPL Token program
///   8. `[optional]` Master edition account
///   9. `[optional]` Asset authorization rules account
pub fn create_metadata<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: CreateMetadataArgs,
) -> ProgramResult {
    match args {
        CreateMetadataArgs::V1 { .. } => create_metadata_v1(program_id, accounts, args),
    }
}

fn create_metadata_v1<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: CreateMetadataArgs,
) -> ProgramResult {
    // get the accounts for the instruction
    let CreateMetadataAccounts::V1 {
        metadata,
        mint,
        mint_authority,
        payer,
        update_authority,
        system_program,
        spl_token_program,
        master_edition,
        authorization_rules,
        ..
    } = args.get_accounts(accounts)?;
    // get the args for the instruction
    let CreateMetadataArgs::V1 {
        ref asset_data,
        decimals,
        max_supply,
    } = args;

    // if the account does not exist, we will allocate a new mint

    if mint.data_is_empty() {
        msg!("Creating mint");
        // mint account must be a signer in the transaction
        if !mint.is_signer {
            return Err(MetadataError::MintIsNotSigner.into());
        }

        invoke(
            &system_instruction::create_account(
                payer.key,
                mint.key,
                Rent::get()?.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            &[payer.clone(), mint.clone()],
        )?;

        let decimals = match asset_data.token_standard {
            // for NonFungible and FungibleAsset variants, we ignore the argument
            // and always use 0 decimals
            Some(TokenStandard::NonFungible)
            | Some(TokenStandard::ProgrammableNonFungible)
            | Some(TokenStandard::FungibleAsset) => 0,
            Some(TokenStandard::Fungible) => match decimals {
                Some(decimals) => decimals,
                // if decimals not provided, use the default
                None => DECIMALS,
            },
            _ => {
                return Err(MetadataError::InvalidTokenStandard.into());
            }
        };

        // initializing the mint account
        invoke(
            &spl_token::instruction::initialize_mint2(
                spl_token_program.key,
                mint.key,
                mint_authority.key,
                Some(mint_authority.key),
                decimals,
            )?,
            &[mint.clone(), mint_authority.clone()],
        )?;
    } else {
        let mint: Mint = assert_initialized(mint, MetadataError::Uninitialized)?;
        // checks that the mint details match the requirements of the
        // token standard selected
        match asset_data.token_standard {
            // for NonFungible and FungibleAsset variants, we ignore the argument
            // and always use 0 decimals
            Some(TokenStandard::NonFungible) | Some(TokenStandard::ProgrammableNonFungible) => {
                if mint.decimals > 0 || mint.supply > 1 {
                    return Err(MetadataError::InvalidMintForTokenStandard.into());
                }
            }
            Some(TokenStandard::FungibleAsset) => {
                if mint.decimals > 0 {
                    return Err(MetadataError::InvalidMintForTokenStandard.into());
                }
            }
            _ => { /* nothing to check */ }
        };
    }

    // creates the metadata account

    process_create_metadata_accounts_logic(
        program_id,
        CreateMetadataAccountsLogicArgs {
            metadata_account_info: metadata,
            mint_info: mint,
            mint_authority_info: mint_authority,
            payer_account_info: payer,
            update_authority_info: update_authority,
            system_account_info: system_program,
        },
        asset_data.as_data(),
        false,
        asset_data.is_mutable,
        false,
        true,
        asset_data.collection_details.clone(),
    )?;

    // creates the metadata account (only for NonFungible assets)

    if let Some(master_edition) = master_edition {
        match asset_data.token_standard {
            Some(TokenStandard::NonFungible) | Some(TokenStandard::ProgrammableNonFungible) => {
                create_master_edition(
                    program_id,
                    master_edition,
                    mint,
                    update_authority,
                    mint_authority,
                    payer,
                    metadata,
                    spl_token_program,
                    system_program,
                    max_supply,
                )?
            }
            _ => { /* noting to do */ }
        }
    }

    let mut asset_metadata = Metadata::from_account_info(metadata)?;
    asset_metadata.token_standard = asset_data.token_standard;

    if let Some(config) = &asset_data.programmable_config {
        if let Some(authorization_rules) = authorization_rules {
            if !cmp_pubkeys(&config.rule_set, authorization_rules.key)
                || authorization_rules.data_is_empty()
            {
                return Err(MetadataError::InvalidAuthorizationRules.into());
            }
            asset_metadata.programmable_config = Some(config.clone());
        }
    }

    // saves the state
    asset_metadata.save(&mut metadata.try_borrow_mut_data()?)?;

    Ok(())
}

enum CreateMetadataAccounts<'a> {
    V1 {
        metadata: &'a AccountInfo<'a>,
        mint: &'a AccountInfo<'a>,
        mint_authority: &'a AccountInfo<'a>,
        payer: &'a AccountInfo<'a>,
        update_authority: &'a AccountInfo<'a>,
        system_program: &'a AccountInfo<'a>,
        _sysvars: &'a AccountInfo<'a>,
        spl_token_program: &'a AccountInfo<'a>,
        master_edition: Option<&'a AccountInfo<'a>>,
        authorization_rules: Option<&'a AccountInfo<'a>>,
    },
}

impl CreateMetadataArgs {
    fn get_accounts<'a>(
        &self,
        accounts: &'a [AccountInfo<'a>],
    ) -> Result<CreateMetadataAccounts<'a>, ProgramError> {
        let account_info_iter = &mut accounts.iter();

        match *self {
            CreateMetadataArgs::V1 { .. } => {
                let metadata = next_account_info(account_info_iter)?;
                let mint = next_account_info(account_info_iter)?;
                let mint_authority = next_account_info(account_info_iter)?;
                let payer = next_account_info(account_info_iter)?;
                let update_authority = next_account_info(account_info_iter)?;
                let system_program = next_account_info(account_info_iter)?;
                let _sysvars = next_account_info(account_info_iter)?;
                let spl_token_program = next_account_info(account_info_iter)?;

                let master_edition =
                    if let Ok(master_edition) = next_account_info(account_info_iter) {
                        Some(master_edition)
                    } else {
                        None
                    };

                let authorization_rules =
                    if let Ok(authorization_rules) = next_account_info(account_info_iter) {
                        Some(authorization_rules)
                    } else {
                        None
                    };

                Ok(CreateMetadataAccounts::V1 {
                    authorization_rules,
                    master_edition,
                    metadata,
                    mint,
                    mint_authority,
                    payer,
                    spl_token_program,
                    system_program,
                    _sysvars,
                    update_authority,
                })
            }
        }
    }
}
