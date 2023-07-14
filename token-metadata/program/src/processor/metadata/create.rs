use mpl_utils::assert_initialized;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke, program_pack::Pack,
    pubkey::Pubkey, rent::Rent, system_instruction, sysvar::Sysvar,
};
use spl_token::{native_mint::DECIMALS, state::Mint};

use crate::{
    error::MetadataError,
    instruction::{Context, Create, CreateArgs},
    state::{
        Metadata, ProgrammableConfig, TokenMetadataAccount, TokenStandard, MAX_MASTER_EDITION_LEN,
        TOKEN_STANDARD_INDEX,
    },
    utils::{
        create_master_edition,
        fee::{levy, set_fee_flag, LevyArgs},
        process_create_metadata_accounts_logic, CreateMetadataAccountsLogicArgs,
    },
};

/// Create the associated metadata accounts for a mint.
///
/// The instruction will also initialize the mint if the account does not
/// exist. For `NonFungible` assets, a `master_edition` account is required.
pub fn create<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: CreateArgs,
) -> ProgramResult {
    let context = Create::to_context(accounts)?;

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
        print_supply,
    } = args;

    // cannot create non-fungible editions on this instruction
    if matches!(
        asset_data.token_standard,
        TokenStandard::NonFungibleEdition | TokenStandard::ProgrammableNonFungibleEdition
    ) {
        return Err(MetadataError::InvalidTokenStandard.into());
    }

    // Levy fees first, to fund the metadata account with rent + fee amount.
    levy(LevyArgs {
        payer_account_info: ctx.accounts.payer_info,
        token_metadata_pda_info: ctx.accounts.metadata_info,
    })?;

    // if the account does not exist, we will allocate a new mint
    if ctx.accounts.mint_info.data_is_empty() {
        // mint account must be a signer in the transaction
        if !ctx.accounts.mint_info.is_signer {
            return Err(MetadataError::MintIsNotSigner.into());
        }

        msg!("Init mint");

        invoke(
            &system_instruction::create_account(
                ctx.accounts.payer_info.key,
                ctx.accounts.mint_info.key,
                Rent::get()?.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::ID,
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
                ctx.accounts.authority_info.key,
                Some(ctx.accounts.authority_info.key),
                decimals,
            )?,
            &[
                ctx.accounts.mint_info.clone(),
                ctx.accounts.authority_info.clone(),
            ],
        )?;
    } else {
        // validates the existing mint account

        let mint: Mint = assert_initialized(ctx.accounts.mint_info, MetadataError::Uninitialized)?;
        // NonFungible assets must have decimals == 0 and supply no greater than 1
        if matches!(
            asset_data.token_standard,
            TokenStandard::NonFungible | TokenStandard::ProgrammableNonFungible
        ) && (mint.decimals > 0 || mint.supply > 1)
        {
            return Err(MetadataError::InvalidMintForTokenStandard.into());
        }
        // Programmable assets must have supply == 0
        if matches!(
            asset_data.token_standard,
            TokenStandard::ProgrammableNonFungible
        ) && (mint.supply > 0)
        {
            return Err(MetadataError::MintSupplyMustBeZero.into());
        }
    }

    // creates the metadata account

    process_create_metadata_accounts_logic(
        program_id,
        CreateMetadataAccountsLogicArgs {
            metadata_account_info: ctx.accounts.metadata_info,
            mint_info: ctx.accounts.mint_info,
            mint_authority_info: ctx.accounts.authority_info,
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
        None,
    )?;

    // creates the master edition account (only for NonFungible assets)

    if matches!(
        asset_data.token_standard,
        TokenStandard::NonFungible | TokenStandard::ProgrammableNonFungible
    ) {
        let print_supply = print_supply.ok_or(MetadataError::MissingPrintSupply)?;

        if let Some(master_edition) = ctx.accounts.master_edition_info {
            create_master_edition(
                program_id,
                master_edition,
                ctx.accounts.mint_info,
                ctx.accounts.update_authority_info,
                ctx.accounts.authority_info,
                ctx.accounts.payer_info,
                ctx.accounts.metadata_info,
                ctx.accounts.spl_token_program_info,
                ctx.accounts.system_program_info,
                print_supply.to_option(),
            )?;

            // for pNFTs, we store the token standard value at the end of the
            // master edition account
            if matches!(
                asset_data.token_standard,
                TokenStandard::ProgrammableNonFungible
            ) {
                let mut data = master_edition.data.borrow_mut();

                if data.len() < MAX_MASTER_EDITION_LEN {
                    return Err(MetadataError::InvalidMasterEditionAccountLength.into());
                }

                data[TOKEN_STANDARD_INDEX] = TokenStandard::ProgrammableNonFungible as u8;
            }
        } else {
            return Err(MetadataError::MissingMasterEditionAccount.into());
        }
    } else if print_supply.is_some() {
        msg!("Ignoring print supply for selected token standard");
    }

    let mut metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    metadata.token_standard = Some(asset_data.token_standard);
    metadata.primary_sale_happened = asset_data.primary_sale_happened;

    // sets the programmable config for programmable assets

    if matches!(
        asset_data.token_standard,
        TokenStandard::ProgrammableNonFungible
    ) {
        metadata.programmable_config = Some(ProgrammableConfig::V1 {
            rule_set: asset_data.rule_set,
        });
    }

    // saves the metadata state
    metadata.save(&mut ctx.accounts.metadata_info.try_borrow_mut_data()?)?;

    // Set fee flag after metadata account is created.
    set_fee_flag(ctx.accounts.metadata_info)
}
