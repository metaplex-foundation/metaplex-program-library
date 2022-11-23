use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    assertions::assert_derivation,
    error::MetadataError,
    pda::find_collection_authority_account,
    state::{
        Collection, CollectionAuthorityRecord, MasterEditionV2, Metadata, TokenMetadataAccount,
        TokenStandard, EDITION, PREFIX,
    },
};

pub fn assert_collection_update_is_valid(
    edition: bool,
    existing: &Option<Collection>,
    incoming: &Option<Collection>,
) -> Result<(), ProgramError> {
    let is_incoming_verified_true = incoming.is_some() && incoming.as_ref().unwrap().verified;

    // If incoming verified is true. Confirm incoming and existing are identical
    let is_incoming_data_valid = !is_incoming_verified_true
        || (existing.is_some()
            && incoming.as_ref().unwrap().verified == existing.as_ref().unwrap().verified
            && incoming.as_ref().unwrap().key == existing.as_ref().unwrap().key);

    if !is_incoming_data_valid && !edition {
        // Never allow a collection to be verified outside of verify_collection instruction
        return Err(MetadataError::CollectionCannotBeVerifiedInThisInstruction.into());
    }
    Ok(())
}

pub fn assert_is_collection_delegated_authority(
    authority_record: &AccountInfo,
    collection_authority: &Pubkey,
    mint: &Pubkey,
) -> Result<u8, ProgramError> {
    let (pda, bump) = find_collection_authority_account(mint, collection_authority);
    if pda != *authority_record.key {
        return Err(MetadataError::DerivedKeyInvalid.into());
    }
    Ok(bump)
}

pub fn assert_has_collection_authority(
    collection_authority_info: &AccountInfo,
    collection_data: &Metadata,
    mint: &Pubkey,
    delegate_collection_authority_record: Option<&AccountInfo>,
) -> Result<(), ProgramError> {
    // Mint is the correct one for the metadata account.
    if collection_data.mint != *mint {
        return Err(MetadataError::MintMismatch.into());
    }

    if let Some(collection_authority_record) = delegate_collection_authority_record {
        let bump = assert_is_collection_delegated_authority(
            collection_authority_record,
            collection_authority_info.key,
            mint,
        )?;
        let data = collection_authority_record.try_borrow_data()?;
        if data.len() == 0 {
            return Err(MetadataError::InvalidCollectionUpdateAuthority.into());
        }
        let record = CollectionAuthorityRecord::from_bytes(&data)?;
        if record.bump != bump {
            return Err(MetadataError::InvalidCollectionUpdateAuthority.into());
        }
        match record.update_authority {
            Some(update_authority) => {
                if update_authority != collection_data.update_authority {
                    return Err(MetadataError::InvalidCollectionUpdateAuthority.into());
                }
            }
            None => return Err(MetadataError::InvalidCollectionUpdateAuthority.into()),
        }
    } else if collection_data.update_authority != *collection_authority_info.key {
        return Err(MetadataError::InvalidCollectionUpdateAuthority.into());
    }
    Ok(())
}

pub fn assert_collection_verify_is_valid(
    member_collection: &Option<Collection>,
    collection_data: &Metadata,
    collection_mint: &AccountInfo,
    edition_account_info: &AccountInfo,
) -> Result<(), ProgramError> {
    match member_collection {
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

    assert_derivation(
        &crate::id(),
        edition_account_info,
        &[
            PREFIX.as_bytes(),
            crate::id().as_ref(),
            collection_data.mint.as_ref(),
            EDITION.as_bytes(),
        ],
    )
    .map_err(|_| MetadataError::CollectionMasterEditionAccountInvalid)?;

    assert_master_edition(collection_data, edition_account_info)?;
    Ok(())
}

pub fn assert_master_edition(
    collection_data: &Metadata,
    edition_account_info: &AccountInfo,
) -> Result<(), ProgramError> {
    let edition = MasterEditionV2::from_account_info(edition_account_info)
        .map_err(|_err: ProgramError| MetadataError::CollectionMustBeAUniqueMasterEdition)?;
    if collection_data.token_standard != Some(TokenStandard::NonFungible)
        || edition.max_supply != Some(0)
    {
        return Err(MetadataError::CollectionMustBeAUniqueMasterEdition.into());
    }
    Ok(())
}
