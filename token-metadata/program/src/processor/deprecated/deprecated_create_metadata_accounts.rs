use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::{
    state::{Data, DataV2},
    utils::{process_create_metadata_accounts_logic, CreateMetadataAccountsLogicArgs},
};

pub fn process_deprecated_create_metadata_accounts<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    data: Data,
    is_mutable: bool,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let metadata_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let payer_account_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;

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
        DataV2 {
            name: data.name,
            uri: data.uri,
            symbol: data.symbol,
            creators: data.creators,
            seller_fee_basis_points: data.seller_fee_basis_points,
            collection: None,
            uses: None,
        },
        false,
        is_mutable,
        false,
        false,
        None, // Does not support collection parents.
    )
}
