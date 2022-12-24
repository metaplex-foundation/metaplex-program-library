use mpl_utils::{assert_initialized, cmp_pubkeys};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke, program_pack::Pack,
    pubkey::Pubkey, rent::Rent, system_instruction, sysvar::Sysvar,
};
use spl_token::{native_mint::DECIMALS, state::Mint};

use crate::{
    error::MetadataError,
    instruction::{Context, Create, CreateArgs},
    state::{Metadata, TokenMetadataAccount, TokenStandard},
    utils::{
        create_master_edition, process_create_metadata_accounts_logic,
        CreateMetadataAccountsLogicArgs,
    },
};

/// Create the associated metadata accounts for mint.
///
/// The instruction will also initialize the mint if the account does not
/// exist. For `Programmable*` assets, if authorization rules are specified,
/// the instruction will check if the account exists.
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
pub fn create<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: CreateArgs,
) -> ProgramResult {
    let context = Create::as_context(accounts)?;

    match args {
        CreateArgs::V1 { .. } => create_v1(program_id, context, args),
    }
}

/// V1 implementation of the create instruction.
fn create_v1(program_id: &Pubkey, ctx: Context<Create>, args: CreateArgs) -> ProgramResult {
    // get the args for the instruction
    let CreateArgs::V1 {
        ref asset_data,
        decimals,
        max_supply,
    } = args;

    // if the account does not exist, we will allocate a new mint

    if ctx.accounts.mint_info.data_is_empty() {
        // mint account must be a signer in the transaction
        if !ctx.accounts.mint_info.is_signer {
            return Err(MetadataError::MintIsNotSigner.into());
        }

        invoke(
            &system_instruction::create_account(
                ctx.accounts.payer_info.key,
                ctx.accounts.mint_info.key,
                Rent::get()?.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            &[
                ctx.accounts.payer_info.clone(),
                ctx.accounts.mint_info.clone(),
            ],
        )?;

        let decimals = match asset_data.token_standard {
            // for NonFungible variants, we ignore the argument and
            // always use 0 decimals
            TokenStandard::NonFungible | TokenStandard::ProgrammableNonFungible => 0,
            // for Fungile variants, we either use the specified decimals or the default
            // DECIMALS from spl-token
            TokenStandard::FungibleAsset | TokenStandard::Fungible => match decimals {
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
                ctx.accounts.spl_token_program_info.key,
                ctx.accounts.mint_info.key,
                ctx.accounts.mint_authority_info.key,
                Some(ctx.accounts.mint_authority_info.key),
                decimals,
            )?,
            &[
                ctx.accounts.mint_info.clone(),
                ctx.accounts.mint_authority_info.clone(),
            ],
        )?;
    } else {
        let mint: Mint = assert_initialized(ctx.accounts.mint_info, MetadataError::Uninitialized)?;
        // NonFungible asset must have decimals = 0 and supply no greater than 1
        if matches!(
            asset_data.token_standard,
            TokenStandard::NonFungible | TokenStandard::ProgrammableNonFungible
        ) && (mint.decimals > 0 || mint.supply > 1)
        {
            return Err(MetadataError::InvalidMintForTokenStandard.into());
        }
    }

    // creates the metadata account

    process_create_metadata_accounts_logic(
        program_id,
        CreateMetadataAccountsLogicArgs {
            metadata_account_info: ctx.accounts.metadata_info,
            mint_info: ctx.accounts.mint_info,
            mint_authority_info: ctx.accounts.mint_authority_info,
            payer_account_info: ctx.accounts.payer_info,
            update_authority_info: ctx.accounts.update_authority_info,
            system_account_info: ctx.accounts.system_program_info,
        },
        asset_data.as_data_v2(),
        false,
        asset_data.is_mutable,
        false,
        true,
        asset_data.collection_details.clone(),
    )?;

    // creates the master edition account (only for NonFungible assets)

    if matches!(
        asset_data.token_standard,
        TokenStandard::NonFungible | TokenStandard::ProgrammableNonFungible
    ) {
        if let Some(master_edition) = ctx.accounts.master_edition_info {
            create_master_edition(
                program_id,
                master_edition,
                ctx.accounts.mint_info,
                ctx.accounts.update_authority_info,
                ctx.accounts.mint_authority_info,
                ctx.accounts.payer_info,
                ctx.accounts.metadata_info,
                ctx.accounts.spl_token_program_info,
                ctx.accounts.system_program_info,
                max_supply,
            )?;
        } else {
            return Err(MetadataError::InvalidMasterEdition.into());
        }
    }

    let mut metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    metadata.token_standard = Some(asset_data.token_standard);

    // sets the programmable config (if present) for programmable assets

    if matches!(
        asset_data.token_standard,
        TokenStandard::ProgrammableNonFungible
    ) {
        if let Some(config) = &asset_data.programmable_config {
            if let Some(authorization_rules) = ctx.accounts.authorization_rules_info {
                if !cmp_pubkeys(&config.rule_set, authorization_rules.key)
                    || authorization_rules.data_is_empty()
                {
                    return Err(MetadataError::InvalidAuthorizationRules.into());
                }
                metadata.programmable_config = Some(config.clone());
            }
        }
    }

    // saves the state
    metadata.save(&mut ctx.accounts.metadata_info.try_borrow_mut_data()?)?;

    Ok(())
}
