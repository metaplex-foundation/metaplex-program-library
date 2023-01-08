use mpl_utils::assert_signer;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    system_program,
};

use crate::{
    assertions::{
        assert_owned_by, collection::assert_has_collection_authority,
        metadata::assert_metadata_derivation,
    },
    error::MetadataError,
    state::{Metadata, TokenMetadataAccount},
    utils::clean_write_metadata,
};

pub fn unverify_collection(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let metadata_info = next_account_info(account_info_iter)?;
    let collection_authority_info = next_account_info(account_info_iter)?;
    let collection_mint_info = next_account_info(account_info_iter)?;
    let collection_metadata_info = next_account_info(account_info_iter)?;
    let edition_account_info = next_account_info(account_info_iter)?;

    let using_delegated_collection_authority = accounts.len() == 6;

    // Account validation.
    assert_owned_by(metadata_info, program_id)?;
    assert_signer(collection_authority_info)?;
    assert_owned_by(collection_mint_info, &spl_token::id())?;

    // Deserialize the collection item metadata.
    let mut metadata = Metadata::from_account_info(metadata_info)?;

    // First, if there's no collection set, we can just short-circuit
    // since there's nothing to unverify.
    if metadata.collection.is_none() {
        return Ok(());
    }

    // Unwrap ok because of check above.
    let collection = metadata.collection.clone().unwrap();

    // Short-circuit if it's already unverified.
    if !collection.verified {
        return Ok(());
    }

    // The collection parent must be the actual parent of the
    // collection item.
    if collection.key != *collection_mint_info.key {
        return Err(MetadataError::NotAMemberOfCollection.into());
    }

    // We need to ensure the metadata is derived from the mint so
    // someone cannot pass in a burned metadata account associated with
    // a different mint.
    assert_metadata_derivation(program_id, collection_metadata_info, collection_mint_info)?;

    // Check if the collection metadata account is burned. If it is,
    // there's no sized data to update and the user can simply unverify
    // the NFT.
    //
    // This check needs to happen before the program owned check.
    let parent_burned = collection_metadata_info.data_is_empty()
        && collection_metadata_info.owner == &system_program::ID;

    if parent_burned {
        // If the parent is burned, we need to check that the authority
        // is the update authority on the item metadata and then we can
        // just unverify the NFT and return.
        //
        // Collection Delegates for burned collection parents should not be
        // respected as there's currently no way to revoke them.

        if metadata.update_authority != *collection_authority_info.key {
            return Err(MetadataError::UpdateAuthorityIncorrect.into());
        }

        metadata.collection.as_mut().unwrap().verified = false;
        clean_write_metadata(&mut metadata, metadata_info)?;
        return Ok(());
    }

    // If the parent is not burned, we need to ensure the collection
    // metadata and edition accounts are owned by the token metadata program.
    assert_owned_by(collection_metadata_info, program_id)?;
    assert_owned_by(edition_account_info, program_id)?;

    // Now we can deserialize the collection metadata account.
    let collection_metadata = Metadata::from_account_info(collection_metadata_info)?;

    // This handler can only unverify non-sized NFTs
    if collection_metadata.collection_details.is_some() {
        return Err(MetadataError::SizedCollection.into());
    }

    if using_delegated_collection_authority {
        let collection_authority_record = next_account_info(account_info_iter)?;
        assert_has_collection_authority(
            collection_authority_info,
            &collection_metadata,
            collection_mint_info.key,
            Some(collection_authority_record),
        )?;
    } else {
        assert_has_collection_authority(
            collection_authority_info,
            &collection_metadata,
            collection_mint_info.key,
            None,
        )?;
    }

    // Unverify and update the metadata
    metadata.collection.as_mut().unwrap().verified = false;
    clean_write_metadata(&mut metadata, metadata_info)?;

    Ok(())
}
