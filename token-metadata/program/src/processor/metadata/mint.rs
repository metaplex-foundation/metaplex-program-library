use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_token::native_mint::DECIMALS;

use crate::{
    instruction::MintArgs,
    state::TokenStandard,
    utils::{
        create_master_edition, process_create_metadata_accounts_logic,
        CreateMetadataAccountsLogicArgs,
    },
};

/// Mint a new asset and associated metadata accounts.
///
/// # Accounts:
///
///   0. `[writable]` Token account
///   1. `[writable]` Metadata account
///   2. `[]` Mint account (signer when account is empty)
///   3. `[signer]` Mint authority
///   4. `[signer]` Payer
///   5. `[signer]` Update authority
///   6. `[]` System program
///   7. `[]` Instructions sysvar account
///   8. `[]` SPL Token program
///   9. `[]` SPL Associated Token Account program
///   10. `[optional]` Master edition account
///   11. `[optional]` Asset authorization rules account
pub fn mint<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: MintArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let token_info = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let _sysvars = next_account_info(account_info_iter)?;
    let spl_token_program_info = next_account_info(account_info_iter)?;
    let _spl_ata_program_info = next_account_info(account_info_iter)?;

    // if the account does not exist, we will allocate a new mint
    if mint_info.data_is_empty() {
        // mint account must be a signer in the transaction
        if !mint_info.is_signer {
            // report error
        }

        // creating the mint account
        invoke(
            &system_instruction::create_account(
                payer_info.key,
                mint_info.key,
                Rent::get()?.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            &[payer_info.clone(), mint_info.clone()],
        )?;

        let token_standard = match &args {
            MintArgs::V1(args) => args.token_standard.clone(),
        };

        let decimals = match token_standard {
            Some(TokenStandard::NonFungible)
            | Some(TokenStandard::NonFungibleEdition)
            | Some(TokenStandard::ProgrammableNonFungible) => 0,
            _ => DECIMALS,
        };

        // initializing the mint account
        invoke(
            &spl_token::instruction::initialize_mint2(
                spl_token_program_info.key,
                mint_info.key,
                mint_authority_info.key,
                Some(mint_authority_info.key),
                decimals,
            )?,
            &[mint_info.clone(), mint_authority_info.clone()],
        )?;
    }

    if token_info.data_is_empty() {
        // creating the associated token account
        invoke(
            &spl_associated_token_account::instruction::create_associated_token_account(
                payer_info.key,
                payer_info.key,
                mint_info.key,
                &spl_token::ID,
            ),
            &[payer_info.clone(), mint_info.clone(), token_info.clone()],
        )?;

        // mints 1 token
        invoke(
            &spl_token::instruction::mint_to(
                spl_token_program_info.key,
                mint_info.key,
                token_info.key,
                payer_info.key,
                &[],
                1,
            )?,
            &[mint_info.clone(), token_info.clone(), payer_info.clone()],
        )?;
    } else {
        // validate the derivation of the token account
    }

    let (data, is_mutable, collection_details) = match &args {
        MintArgs::V1(args) => (
            args.as_data(),
            args.is_mutable,
            args.collection_details.clone(),
        ),
    };

    // creates the metadata account

    process_create_metadata_accounts_logic(
        program_id,
        CreateMetadataAccountsLogicArgs {
            metadata_account_info: metadata_info,
            mint_info,
            mint_authority_info,
            payer_account_info: payer_info,
            update_authority_info,
            system_account_info: system_program_info,
        },
        data,
        false,
        is_mutable,
        false,
        true,
        collection_details,
    )?;

    if let Ok(master_edition_info) = next_account_info(account_info_iter) {
        msg!("Metadata data len: {}", metadata_info.data_len());
        create_master_edition(
            program_id,
            master_edition_info,
            mint_info,
            update_authority_info,
            mint_authority_info,
            payer_info,
            metadata_info,
            spl_token_program_info,
            system_program_info,
            None,
        )?;
    }

    Ok(())
}
