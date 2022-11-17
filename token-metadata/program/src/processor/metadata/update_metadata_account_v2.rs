use borsh::{BorshDeserialize, BorshSerialize};
pub use instruction::*;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::{
    assertions::{
        assert_owned_by,
        collection::assert_collection_update_is_valid,
        metadata::{assert_data_valid, assert_update_authority_is_correct},
        uses::assert_valid_use,
    },
    deser::clean_write_metadata,
    error::MetadataError,
    instruction::MetadataInstruction,
    state::{DataV2, Metadata, TokenMetadataAccount},
    utils::puff_out_data_fields,
};

mod instruction {
    #[cfg(feature = "serde-feature")]
    use {
        serde::{Deserialize, Serialize},
        serde_with::{As, DisplayFromStr},
    };

    use super::*;

    #[repr(C)]
    #[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
    #[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
    /// Args for update call
    pub struct UpdateMetadataAccountArgsV2 {
        pub data: Option<DataV2>,
        #[cfg_attr(
            feature = "serde-feature",
            serde(with = "As::<Option<DisplayFromStr>>")
        )]
        pub update_authority: Option<Pubkey>,
        pub primary_sale_happened: Option<bool>,
        pub is_mutable: Option<bool>,
    }

    // update metadata account v2 instruction
    pub fn update_metadata_accounts_v2(
        program_id: Pubkey,
        metadata_account: Pubkey,
        update_authority: Pubkey,
        new_update_authority: Option<Pubkey>,
        data: Option<DataV2>,
        primary_sale_happened: Option<bool>,
        is_mutable: Option<bool>,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(metadata_account, false),
                AccountMeta::new_readonly(update_authority, true),
            ],
            data: MetadataInstruction::UpdateMetadataAccountV2(UpdateMetadataAccountArgsV2 {
                data,
                update_authority: new_update_authority,
                primary_sale_happened,
                is_mutable,
            })
            .try_to_vec()
            .unwrap(),
        }
    }
}

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
