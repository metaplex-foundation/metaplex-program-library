use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::MetadataError,
    state::{Collection, DataV2, MasterEditionV2, Metadata, TokenStandard},
};

pub fn assert_collection_update_is_valid(
    _existing: &Option<Collection>,
    incoming: &Option<Collection>,
) -> Result<(), ProgramError> {
    if incoming.is_some() && incoming.as_ref().unwrap().verified == true {
        // Never allow a collection to be verified outside of verify_collection instruction
        return Err(MetadataError::CollectionCannotBeVerifiedInThisInstruction.into());
    }
    Ok(())
}

pub fn assert_collection_verify_if_valid(
    collection_member: &Metadata,
    collection_data: &Metadata,
    collection_mint: &AccountInfo,
    collection_authority_info: &AccountInfo,
    edition_account_info: &AccountInfo,
) -> Result<(), ProgramError> {
    match &collection_member.collection {
        Some(collection) => {
            if collection.key != *collection_mint.key
                || collection_data.mint != *collection_mint.key
            {
                return Err(MetadataError::CollectionNotFound.into());
            }
            if collection_data.update_authority != *collection_authority_info.key {
                return Err(MetadataError::InvalidCollectionUpdateAuthority.into());
            }
        }
        None => {
            return Err(MetadataError::CollectionNotFound.into());
        }
    }
    let edition = MasterEditionV2::from_account_info(edition_account_info)
        .map_err(|_err: ProgramError| MetadataError::CollectionMustBeAUniqueMasterEdition)?;
    if collection_data.token_standard != Some(TokenStandard::NonFungible)
        || edition.max_supply != Some(0)
    {
        return Err(MetadataError::CollectionMustBeAUniqueMasterEdition.into());
    }
    Ok(())
}
