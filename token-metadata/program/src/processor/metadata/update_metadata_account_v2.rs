use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::{
    assertions::{
        assert_owned_by,
        collection::assert_collection_update_is_valid,
        metadata::{assert_data_valid, assert_update_authority_is_correct},
        uses::assert_valid_use,
    },
    error::MetadataError,
    state::{DataV2, Metadata, TokenMetadataAccount},
    utils::{clean_write_metadata, puff_out_data_fields},
};

// Update existing account instruction
pub fn process_update_metadata_accounts_v2(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    optional_data: Option<DataV2>,
    update_authority: Option<Pubkey>,
    primary_sale_happened: Option<bool>,
    is_mutable: Option<bool>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let metadata_account_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let mut metadata = Metadata::from_account_info(metadata_account_info)?;

    assert_owned_by(metadata_account_info, program_id)?;
    assert_update_authority_is_correct(&metadata, update_authority_info)?;

    if let Some(data) = optional_data {
        if metadata.is_mutable {
            let compatible_data = data.to_v1();
            assert_data_valid(
                &compatible_data,
                update_authority_info.key,
                &metadata,
                false,
                update_authority_info.is_signer,
            )?;
            metadata.data = compatible_data;
            // If the user passes in Collection data, only allow updating if it's unverified
            // or if it exactly matches the existing collection info.
            // If the user passes in None for the Collection data then only set it if it's unverified.
            if data.collection.is_some() {
                assert_collection_update_is_valid(false, &metadata.collection, &data.collection)?;
                metadata.collection = data.collection;
            } else if let Some(collection) = metadata.collection.as_ref() {
                // Can't change a verified collection in this command.
                if collection.verified {
                    return Err(MetadataError::CannotUpdateVerifiedCollection.into());
                }
                // If it's unverified, it's ok to set to None.
                metadata.collection = data.collection;
            }
            // If already None leave it as None.
            assert_valid_use(&data.uses, &metadata.uses)?;
            metadata.uses = data.uses;
        } else {
            return Err(MetadataError::DataIsImmutable.into());
        }
    }

    if let Some(val) = update_authority {
        metadata.update_authority = val;
    }

    if let Some(val) = primary_sale_happened {
        // If received val is true, flip to true.
        if val || !metadata.primary_sale_happened {
            metadata.primary_sale_happened = val
        } else {
            return Err(MetadataError::PrimarySaleCanOnlyBeFlippedToTrue.into());
        }
    }

    if let Some(val) = is_mutable {
        // If received value is false, flip to false.
        if !val || metadata.is_mutable {
            metadata.is_mutable = val
        } else {
            return Err(MetadataError::IsMutableCanOnlyBeFlippedToFalse.into());
        }
    }

    puff_out_data_fields(&mut metadata);
    clean_write_metadata(&mut metadata, metadata_account_info)?;
    Ok(())
}
