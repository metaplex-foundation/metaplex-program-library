use mpl_utils::assert_signer;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::{
    assertions::{
        assert_owned_by,
        collection::{assert_collection_verify_is_valid, assert_has_collection_authority},
    },
    error::MetadataError,
    state::{Metadata, TokenMetadataAccount},
    utils::clean_write_metadata,
};

pub fn verify_collection(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let metadata_info = next_account_info(account_info_iter)?;
    let collection_authority_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let collection_mint = next_account_info(account_info_iter)?;
    let collection_info = next_account_info(account_info_iter)?;
    let edition_account_info = next_account_info(account_info_iter)?;

    assert_signer(collection_authority_info)?;
    assert_signer(payer_info)?;

    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(collection_info, program_id)?;
    assert_owned_by(collection_mint, &spl_token::ID)?;
    assert_owned_by(edition_account_info, program_id)?;

    let mut metadata = Metadata::from_account_info(metadata_info)?;
    let collection_metadata = Metadata::from_account_info(collection_info)?;

    assert_collection_verify_is_valid(
        &metadata.collection,
        &collection_metadata,
        collection_mint,
        edition_account_info,
    )?;

    let delegated_collection_authority_opt = account_info_iter.next();

    assert_has_collection_authority(
        collection_authority_info,
        &collection_metadata,
        collection_mint.key,
        delegated_collection_authority_opt,
    )?;

    // This handler can only verify non-sized NFTs
    if collection_metadata.collection_details.is_some() {
        return Err(MetadataError::SizedCollection.into());
    }

    // If the NFT has collection data, we set it to be verified
    if let Some(collection) = &mut metadata.collection {
        collection.verified = true;
        clean_write_metadata(&mut metadata, metadata_info)?;
    }

    Ok(())
}
