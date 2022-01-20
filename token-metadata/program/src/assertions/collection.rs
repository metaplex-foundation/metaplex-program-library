use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::MetadataError,
    pda::find_collection_authority_account,
    state::{
        Collection, DataV2, MasterEditionV2, Metadata, TokenStandard, COLLECTION_AUTHORITY, PREFIX,
    },
    utils::assert_derivation,
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

pub fn assert_is_collection_delegated_authority(
    authority_record: &AccountInfo,
    collection_authority: &Pubkey,
    mint: &Pubkey,
) -> Result<(), ProgramError> {
    let (pda, _) = find_collection_authority_account(mint, collection_authority);
    if pda != *authority_record.key {
        return Err(MetadataError::DerivedKeyInvalid.into());
    }
    Ok(())
}

pub fn assert_has_collection_authority(
    collection_authority_info: &AccountInfo,
    collection_data: &Metadata,
    mint: &Pubkey,
    delegate_collection_authority_record: Option<&AccountInfo>,
) -> Result<(), ProgramError> {
    if delegate_collection_authority_record.is_some() {
        assert_is_collection_delegated_authority(
            delegate_collection_authority_record.unwrap(),
            collection_authority_info.key,
            mint,
        )?;
        if delegate_collection_authority_record
            .unwrap()
            .try_data_is_empty()?
            || delegate_collection_authority_record.unwrap().data.borrow()[0] == 0
        {
            return Err(MetadataError::InvalidCollectionUpdateAuthority.into());
        }
    } else {
        if collection_data.update_authority != *collection_authority_info.key {
            return Err(MetadataError::InvalidCollectionUpdateAuthority.into());
        }
    }
    Ok(())
}

pub fn assert_collection_verify_is_valid(
    collection_member: &Metadata,
    collection_data: &Metadata,
    collection_mint: &AccountInfo,
    edition_account_info: &AccountInfo,
) -> Result<(), ProgramError> {
    match &collection_member.collection {
        Some(collection) => {
            if collection.key != *collection_mint.key
                || collection_data.mint != *collection_mint.key
            {
                return Err(MetadataError::CollectionNotFound.into());
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
