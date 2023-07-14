use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::{
    state::{CollectionDetails, DataV2},
    utils::{
        fee::{levy, set_fee_flag, LevyArgs},
        process_create_metadata_accounts_logic, CreateMetadataAccountsLogicArgs,
    },
};

pub fn process_create_metadata_accounts_v3<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    data: DataV2,
    is_mutable: bool,
    collection_details: Option<CollectionDetails>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let metadata_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let payer_account_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;

    // Levy fees first, to fund the metadata account with rent + fee amount.
    levy(LevyArgs {
        payer_account_info,
        token_metadata_pda_info: metadata_account_info,
    })?;

    process_create_metadata_accounts_logic(
        program_id,
        CreateMetadataAccountsLogicArgs {
            metadata_account_info,
            mint_info,
            mint_authority_info,
            payer_account_info,
            update_authority_info,
            system_account_info,
        },
        data,
        false,
        is_mutable,
        false,
        true,
        collection_details,
        None,
    )?;

    // Set fee flag after metadata account is created.
    set_fee_flag(metadata_account_info)
}
