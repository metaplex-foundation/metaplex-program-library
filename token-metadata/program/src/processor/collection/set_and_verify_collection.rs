use borsh::BorshSerialize;
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
};

pub fn set_and_verify_collection(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let metadata_info = next_account_info(account_info_iter)?;
    let collection_authority_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let update_authority = next_account_info(account_info_iter)?;
    let collection_mint = next_account_info(account_info_iter)?;
    let collection_info = next_account_info(account_info_iter)?;
    let edition_account_info = next_account_info(account_info_iter)?;
    let using_delegated_collection_authority = accounts.len() == 8;

    assert_signer(collection_authority_info)?;
    assert_signer(payer_info)?;

    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(collection_info, program_id)?;
    assert_owned_by(collection_mint, &spl_token::id())?;
    assert_owned_by(edition_account_info, program_id)?;

    let mut metadata = Metadata::from_account_info(metadata_info)?;
    let collection_data = Metadata::from_account_info(collection_info)?;
    if metadata.update_authority != *update_authority.key
        || metadata.update_authority != collection_data.update_authority
    {
        return Err(MetadataError::UpdateAuthorityIncorrect.into());
    }

    // If it's a verified item and the user is trying to move it to a new collection,
    // they must unverify first, in case it belongs to a sized collection.
    if let Some(collection) = metadata.collection {
        if collection.key != *collection_mint.key && collection.verified {
            return Err(MetadataError::MustUnverify.into());
        }
    }

    if using_delegated_collection_authority {
        let collection_authority_record = next_account_info(account_info_iter)?;
        assert_has_collection_authority(
            collection_authority_info,
            &collection_data,
            collection_mint.key,
            Some(collection_authority_record),
        )?;
    } else {
        assert_has_collection_authority(
            collection_authority_info,
            &collection_data,
            collection_mint.key,
            None,
        )?;
    }
    metadata.collection = Some(Collection {
        key: *collection_mint.key,
        verified: true,
    });
    assert_collection_verify_is_valid(
        &metadata.collection,
        &collection_data,
        collection_mint,
        edition_account_info,
    )?;

    // This handler can only verify non-sized NFTs
    if collection_data.collection_details.is_some() {
        return Err(MetadataError::SizedCollection.into());
    }

    metadata.serialize(&mut *metadata_info.try_borrow_mut_data()?)?;
    Ok(())
}
