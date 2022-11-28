use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::{
    assertions::{assert_owned_by, collection::assert_has_collection_authority},
    error::MetadataError,
    instruction::SetCollectionSizeArgs,
    state::{CollectionDetails, Metadata, TokenMetadataAccount},
    utils::clean_write_metadata,
};

pub fn set_collection_size(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: SetCollectionSizeArgs,
) -> ProgramResult {
    let size = args.size;

    let account_info_iter = &mut accounts.iter();

    let parent_nft_metadata_account_info = next_account_info(account_info_iter)?;
    let collection_update_authority_account_info = next_account_info(account_info_iter)?;
    let collection_mint_account_info = next_account_info(account_info_iter)?;

    let using_delegated_collection_authority = accounts.len() == 4;

    // Owned by token-metadata program.
    assert_owned_by(parent_nft_metadata_account_info, program_id)?;

    // Mint owned by spl token program.
    assert_owned_by(collection_mint_account_info, &spl_token::id())?;

    let mut metadata = Metadata::from_account_info(parent_nft_metadata_account_info)?;

    // Check that the update authority or delegate is a signer.
    if !collection_update_authority_account_info.is_signer {
        return Err(MetadataError::UpdateAuthorityIsNotSigner.into());
    }

    if using_delegated_collection_authority {
        let collection_authority_record = next_account_info(account_info_iter)?;
        assert_has_collection_authority(
            collection_update_authority_account_info,
            &metadata,
            collection_mint_account_info.key,
            Some(collection_authority_record),
        )?;
    } else {
        assert_has_collection_authority(
            collection_update_authority_account_info,
            &metadata,
            collection_mint_account_info.key,
            None,
        )?;
    }

    // Only unsized collections can have the size set, and only once.
    if metadata.collection_details.is_some() {
        return Err(MetadataError::SizedCollection.into());
    } else {
        metadata.collection_details = Some(CollectionDetails::V1 { size });
    }

    clean_write_metadata(&mut metadata, parent_nft_metadata_account_info)?;
    Ok(())
}
