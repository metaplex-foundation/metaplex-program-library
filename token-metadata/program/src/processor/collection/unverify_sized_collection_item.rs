use mpl_utils::assert_signer;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::{
    assertions::{
        assert_owned_by, collection::assert_has_collection_authority,
        metadata::assert_metadata_derivation,
    },
    error::MetadataError,
    state::{Metadata, TokenMetadataAccount},
    utils::{clean_write_metadata, decrement_collection_size},
};

pub fn unverify_sized_collection_item(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let metadata_info = next_account_info(account_info_iter)?;
    let collection_authority_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let collection_mint_info = next_account_info(account_info_iter)?;
    let collection_metadata_info = next_account_info(account_info_iter)?;
    let _edition_account_info = next_account_info(account_info_iter)?;

    assert_signer(collection_authority_info)?;
    assert_signer(payer_info)?;

    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(collection_mint_info, &spl_token::ID)?;

    let mut metadata = Metadata::from_account_info(metadata_info)?;

    // First, if there's no collection set, we can just short-circuit
    // since there's nothing to unverify.
    let collection = match metadata.collection.as_mut() {
        Some(collection) => collection,
        None => return Ok(()),
    };

    // Short-circuit if it's already unverified.
    if !collection.verified {
        return Ok(());
    }

    // The collection parent must be the actual parent of the
    // collection item.
    if collection.key != *collection_mint_info.key {
        return Err(MetadataError::NotAMemberOfCollection.into());
    }

    assert_metadata_derivation(program_id, collection_metadata_info, collection_mint_info)?;

    // Check if the collection metadata account is burned. If it is,
    // there's no sized data to update and the user can simply unverify
    // the NFT.
    //
    // This check needs to happen before the program owned check.
    let parent_burned =
        collection_metadata_info.data_is_empty() || collection_metadata_info.data.borrow()[0] == 0;

    if parent_burned {
        // If the parent is burned, we need to check that the authority
        // is the update authority on the item metadata.
        //
        // Collection Delegates for burned collection parents should not be
        // respected as there's currently no way to revoke them.

        if metadata.update_authority != *collection_authority_info.key {
            return Err(MetadataError::UpdateAuthorityIncorrect.into());
        }
    } else {
        // If the parent is not burned, we need to ensure the collection
        // metadata and edition accounts are owned by the token metadata program.
        assert_owned_by(collection_metadata_info, program_id)?;

        // Now we can deserialize the collection metadata account.
        let mut collection_metadata = Metadata::from_account_info(collection_metadata_info)?;

        let delegated_collection_authority_opt = account_info_iter.next();

        assert_has_collection_authority(
            collection_authority_info,
            &collection_metadata,
            collection_mint_info.key,
            delegated_collection_authority_opt,
        )?;
        decrement_collection_size(&mut collection_metadata, collection_metadata_info)?;
    }

    collection.verified = false;
    clean_write_metadata(&mut metadata, metadata_info)
}
