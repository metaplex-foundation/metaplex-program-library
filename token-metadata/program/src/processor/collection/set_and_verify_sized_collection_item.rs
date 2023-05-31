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
    state::{Collection, Metadata, TokenMetadataAccount},
    utils::{clean_write_metadata, increment_collection_size},
};

pub fn set_and_verify_sized_collection_item(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let metadata_info = next_account_info(account_info_iter)?;
    let collection_authority_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let update_authority = next_account_info(account_info_iter)?;
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
    let mut collection_metadata = Metadata::from_account_info(collection_info)?;

    // Don't verify already verified items, otherwise we end up with invalid size data.
    if let Some(collection) = metadata.collection {
        if collection.verified {
            return Err(MetadataError::MustUnverify.into());
        }
    }

    if metadata.update_authority != *update_authority.key
        || metadata.update_authority != collection_metadata.update_authority
    {
        return Err(MetadataError::UpdateAuthorityIncorrect.into());
    }

    let delegated_collection_authority_opt = account_info_iter.next();

    assert_has_collection_authority(
        collection_authority_info,
        &collection_metadata,
        collection_mint.key,
        delegated_collection_authority_opt,
    )?;

    metadata.collection = Some(Collection {
        key: *collection_mint.key,
        verified: true,
    });
    assert_collection_verify_is_valid(
        &metadata.collection,
        &collection_metadata,
        collection_mint,
        edition_account_info,
    )?;

    // Update the collection size if this is a valid parent collection NFT.
    increment_collection_size(&mut collection_metadata, collection_info)?;

    clean_write_metadata(&mut metadata, metadata_info)
}
