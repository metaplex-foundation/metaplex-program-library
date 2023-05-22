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

/// Checks whether the collection update is allowed or not based on the `verified` status.
pub fn assert_collection_update_is_valid(
    allow_direct_collection_verified_writes: bool,
    existing: &Option<Collection>,
    incoming: &Option<Collection>,
) -> Result<(), ProgramError> {
    let is_incoming_verified = if let Some(status) = incoming {
        status.verified
    } else {
        false
    };

    let is_existing_verified = if let Some(status) = existing {
        status.verified
    } else {
        false
    };

    let valid_update = if is_incoming_verified {
        // verified: can only update if the details match
        is_existing_verified && (existing.as_ref().unwrap().key == incoming.as_ref().unwrap().key)
    } else {
        // unverified: can only update if existing is unverified
        !is_existing_verified
    };

    // overrule: if we are dealing with an edition or a Bubblegum decompression.
    if !valid_update && !allow_direct_collection_verified_writes {
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
    collection_metadata: &Metadata,
    collection_mint: &AccountInfo,
    edition_account_info: &AccountInfo,
) -> Result<(), ProgramError> {
    match member_collection {
        Some(collection) => {
            if collection.key != *collection_mint.key
                || collection_metadata.mint != *collection_mint.key
            {
                return Err(MetadataError::CollectionNotFound.into());
            }
        }
        None => {
            return Err(MetadataError::CollectionNotFound.into());
        }
    }

    assert_derivation(
        &crate::ID,
        edition_account_info,
        &[
            PREFIX.as_bytes(),
            crate::ID.as_ref(),
            collection_metadata.mint.as_ref(),
            EDITION.as_bytes(),
        ],
    )
    .map_err(|_| MetadataError::CollectionMasterEditionAccountInvalid)?;

    assert_master_edition(collection_metadata, edition_account_info)?;
    Ok(())
}

pub fn assert_master_edition(
    collection_data: &Metadata,
    edition_account_info: &AccountInfo,
) -> Result<(), ProgramError> {
    let edition = MasterEditionV2::from_account_info(edition_account_info)
        .map_err(|_err: ProgramError| MetadataError::CollectionMustBeAUniqueMasterEdition)?;

    match collection_data.token_standard {
        Some(TokenStandard::NonFungible) | Some(TokenStandard::ProgrammableNonFungible) => (),
        _ => return Err(MetadataError::CollectionMustBeAUniqueMasterEdition.into()),
    }

    if edition.max_supply != Some(0) {
        return Err(MetadataError::CollectionMustBeAUniqueMasterEdition.into());
    }
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_assert_collection_update_is_valid() {
        let key_1 = Pubkey::new_unique();
        let key_2 = Pubkey::new_unique();

        // collection 1

        let collection_key1_false = Collection {
            key: key_1,
            verified: false,
        };

        let collection_key1_true = Collection {
            key: key_1,
            verified: true,
        };

        // collection 2

        let collection_key2_false = Collection {
            key: key_2,
            verified: false,
        };

        let collection_key2_true = Collection {
            key: key_2,
            verified: true,
        };

        // [OK] "unverified" same collection details

        assert_collection_update_is_valid(
            false,
            &Some(collection_key1_false.clone()),
            &Some(collection_key1_false.clone()),
        )
        .unwrap();

        // [OK] "verified" same collection details

        assert_collection_update_is_valid(
            false,
            &Some(collection_key1_true.clone()),
            &Some(collection_key1_true.clone()),
        )
        .unwrap();

        // [ERROR] "unverify" collection

        assert_collection_update_is_valid(
            false,
            &Some(collection_key1_true.clone()),
            &Some(collection_key1_false.clone()),
        )
        .unwrap_err();

        // [ERROR] "verify" collection

        assert_collection_update_is_valid(
            false,
            &Some(collection_key1_false.clone()),
            &Some(collection_key1_true.clone()),
        )
        .unwrap_err();

        // [OK] "unverified" update collection details

        assert_collection_update_is_valid(
            false,
            &Some(collection_key1_false.clone()),
            &Some(collection_key2_false.clone()),
        )
        .unwrap();

        // [ERROR] "verified" update collection details

        assert_collection_update_is_valid(
            false,
            &Some(collection_key1_false),
            &Some(collection_key2_true.clone()),
        )
        .unwrap_err();

        // [ERROR] "verified" update collection details

        assert_collection_update_is_valid(
            false,
            &Some(collection_key1_true.clone()),
            &Some(collection_key2_false),
        )
        .unwrap_err();

        // [ERROR] "verified" update collection details

        assert_collection_update_is_valid(
            false,
            &Some(collection_key1_true.clone()),
            &Some(collection_key2_true.clone()),
        )
        .unwrap_err();

        // [OK] "edition" override

        assert_collection_update_is_valid(
            true,
            &Some(collection_key1_true),
            &Some(collection_key2_true),
        )
        .unwrap();
    }
}
