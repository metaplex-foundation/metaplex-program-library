use mpl_utils::assert_signer;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};
use spl_token::state::Account as TokenAccount;

use crate::{
    assertions::{assert_owned_by, metadata::assert_currently_holding},
    instruction::{Burn, Context},
    state::{Metadata, TokenMetadataAccount},
    utils::assert_initialized,
};

use super::nonfungible_edition::burn_nonfungible_edition;

pub fn process_burn_edition_nft<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let metadata_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let print_edition_mint_info = next_account_info(account_info_iter)?;
    let master_edition_mint_info = next_account_info(account_info_iter)?;
    let print_edition_token_info = next_account_info(account_info_iter)?;
    let master_edition_token_info = next_account_info(account_info_iter)?;
    let master_edition_info = next_account_info(account_info_iter)?;
    let print_edition_info = next_account_info(account_info_iter)?;
    let edition_marker_info = next_account_info(account_info_iter)?;
    let spl_token_program_info = next_account_info(account_info_iter)?;

    // Validate accounts
    // Owner is a signer.
    assert_signer(owner_info)?;

    // Owned by token-metadata program.
    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(master_edition_info, program_id)?;
    assert_owned_by(print_edition_info, program_id)?;
    assert_owned_by(edition_marker_info, program_id)?;

    // Owned by spl-token program.
    assert_owned_by(master_edition_mint_info, &spl_token::id())?;
    assert_owned_by(master_edition_token_info, &spl_token::id())?;
    assert_owned_by(print_edition_mint_info, &spl_token::id())?;
    assert_owned_by(print_edition_token_info, &spl_token::id())?;

    let metadata = Metadata::from_account_info(metadata_info)?;
    let _: TokenAccount = assert_initialized(print_edition_token_info)?;

    // Checks:
    // * Metadata is owned by the token-metadata program
    // * Mint is owned by the spl-token program
    // * Token is owned by the spl-token program
    // * Token account is initialized
    // * Token account data owner is 'owner'
    // * Token account belongs to mint
    // * Token account has 1 or more tokens
    // * Mint matches metadata.mint
    assert_currently_holding(
        program_id,
        owner_info,
        metadata_info,
        &metadata,
        print_edition_mint_info,
        print_edition_token_info,
    )?;

    // Contruct our new Burn handler context so we can re-use the same code for both.
    let accounts = Burn {
        authority_info: owner_info,
        collection_metadata_info: None,
        metadata_info,
        edition_info: Some(print_edition_info),
        mint_info: print_edition_mint_info,
        token_info: print_edition_token_info,
        parent_edition_info: Some(master_edition_info),
        parent_mint_info: Some(master_edition_mint_info),
        parent_token_info: Some(master_edition_token_info),
        edition_marker_info: Some(edition_marker_info),
        token_record_info: None,
        // This handler doesn't get system program and sysvars instructions
        // but we need them to create the Burn struct. They are not used in the burn_nonfungible_edition handler.
        system_program_info: spl_token_program_info,
        sysvar_instructions_info: spl_token_program_info,
        spl_token_program_info,
    };
    let context = Context {
        accounts,
        remaining_accounts: vec![],
    };

    burn_nonfungible_edition(&context)

    // // Master Edition token account checks.
    // let master_edition_token_account: Account = assert_initialized(master_edition_token_info)?;

    // if master_edition_token_account.mint != *master_edition_mint_info.key {
    //     return Err(MetadataError::MintMismatch.into());
    // }

    // if master_edition_token_account.amount < 1 {
    //     return Err(MetadataError::NotEnoughTokens.into());
    // }

    // // Master and Print editions are valid PDAs for their given mints.
    // let master_edition_info_path = Vec::from([
    //     PREFIX.as_bytes(),
    //     program_id.as_ref(),
    //     master_edition_mint_info.key.as_ref(),
    //     EDITION.as_bytes(),
    // ]);
    // assert_derivation(program_id, master_edition_info, &master_edition_info_path)
    //     .map_err(|_| MetadataError::InvalidMasterEdition)?;

    // let print_edition_info_path = Vec::from([
    //     PREFIX.as_bytes(),
    //     program_id.as_ref(),
    //     print_edition_mint_info.key.as_ref(),
    //     EDITION.as_bytes(),
    // ]);
    // assert_derivation(program_id, print_edition_info, &print_edition_info_path)
    //     .map_err(|_| MetadataError::InvalidPrintEdition)?;

    // let print_edition = Edition::from_account_info(print_edition_info)?;

    // // Print Edition actually belongs to the master edition.
    // if print_edition.parent != *master_edition_info.key {
    //     return Err(MetadataError::PrintEditionDoesNotMatchMasterEdition.into());
    // }

    // // Which edition marker is this edition in
    // let edition_marker_number = print_edition
    //     .edition
    //     .checked_div(EDITION_MARKER_BIT_SIZE)
    //     .ok_or(MetadataError::NumericalOverflowError)?;
    // let edition_marker_number_str = edition_marker_number.to_string();

    // // Ensure we were passed the correct edition marker PDA.
    // let edition_marker_info_path = Vec::from([
    //     PREFIX.as_bytes(),
    //     program_id.as_ref(),
    //     master_edition_mint_info.key.as_ref(),
    //     EDITION.as_bytes(),
    //     edition_marker_number_str.as_bytes(),
    // ]);
    // assert_derivation(program_id, edition_marker_info, &edition_marker_info_path)
    //     .map_err(|_| MetadataError::InvalidEditionMarker)?;

    // //      **BURN**
    // // Burn the SPL token
    // let params = TokenBurnParams {
    //     mint: print_edition_mint_info.clone(),
    //     source: print_edition_token_info.clone(),
    //     authority: owner_info.clone(),
    //     token_program: spl_token_program_info.clone(),
    //     amount: 1,
    //     authority_signer_seeds: None,
    // };
    // spl_token_burn(params)?;

    // // Close token account.
    // let params = TokenCloseParams {
    //     token_program: spl_token_program_info.clone(),
    //     account: print_edition_token_info.clone(),
    //     destination: owner_info.clone(),
    //     owner: owner_info.clone(),
    //     authority_signer_seeds: None,
    // };
    // spl_token_close(params)?;

    // // Close metadata and edition accounts by transferring rent funds to owner and
    // // zeroing out the data.
    // let metadata_lamports = metadata_info.lamports();
    // **metadata_info.try_borrow_mut_lamports()? = 0;
    // **owner_info.try_borrow_mut_lamports()? = owner_info
    //     .lamports()
    //     .checked_add(metadata_lamports)
    //     .ok_or(MetadataError::NumericalOverflowError)?;

    // let edition_lamports = print_edition_info.lamports();
    // **print_edition_info.try_borrow_mut_lamports()? = 0;
    // **owner_info.try_borrow_mut_lamports()? = owner_info
    //     .lamports()
    //     .checked_add(edition_lamports)
    //     .ok_or(MetadataError::NumericalOverflowError)?;

    // let metadata_data = &mut metadata_info.try_borrow_mut_data()?;
    // let edition_data = &mut print_edition_info.try_borrow_mut_data()?;
    // let edition_data_len = edition_data.len();

    // // Use MAX_METADATA_LEN because it has unused padding, making it longer than current metadata len.
    // sol_memset(metadata_data, 0, MAX_METADATA_LEN);
    // sol_memset(edition_data, 0, edition_data_len);

    // //       **EDITION HOUSEKEEPING**
    // // Set the particular bit for this edition to 0 to allow reprinting,
    // // IF the print edition owner is also the master edition owner.
    // // Otherwise leave the bit set to 1 to disallow reprinting.
    // let mut edition_marker: EditionMarker = EditionMarker::from_account_info(edition_marker_info)?;
    // let owner_is_the_same = *owner_info.key == master_edition_token_account.owner;

    // if owner_is_the_same {
    //     let (index, mask) = EditionMarker::get_index_and_mask(print_edition.edition)?;
    //     edition_marker.ledger[index] ^= mask;
    // }

    // // If the entire edition marker is empty, then we can close the account.
    // // Otherwise, serialize the new edition marker and update the account data.
    // if edition_marker.ledger.iter().all(|i| *i == 0) {
    //     let edition_marker_lamports = edition_marker_info.lamports();
    //     **edition_marker_info.try_borrow_mut_lamports()? = 0;
    //     **owner_info.try_borrow_mut_lamports()? = owner_info
    //         .lamports()
    //         .checked_add(edition_marker_lamports)
    //         .ok_or(MetadataError::NumericalOverflowError)?;

    //     let edition_marker_data = &mut edition_marker_info.try_borrow_mut_data()?;
    //     let edition_marker_data_len = edition_marker_data.len();

    //     sol_memset(edition_marker_data, 0, edition_marker_data_len);
    // } else {
    //     let mut edition_marker_info_data = edition_marker_info.try_borrow_mut_data()?;
    //     edition_marker_info_data[0..].fill(0);
    //     edition_marker.serialize(&mut *edition_marker_info_data)?;
    // }

    // // Decrement the suppply on the master edition now that we've successfully burned a print.
    // let mut master_edition: MasterEditionV2 =
    //     MasterEditionV2::from_account_info(master_edition_info)?;
    // master_edition.supply = master_edition
    //     .supply
    //     .checked_sub(1)
    //     .ok_or(MetadataError::NumericalOverflowError)?;

    // master_edition.save(master_edition_info)?;
}
