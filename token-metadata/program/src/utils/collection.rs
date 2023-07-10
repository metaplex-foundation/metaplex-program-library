use solana_program::msg;

use super::*;
use crate::state::CollectionDetails;

pub fn increment_collection_size(
    metadata: &mut Metadata,
    metadata_info: &AccountInfo,
) -> ProgramResult {
    if let Some(ref details) = metadata.collection_details {
        match details {
            #[allow(deprecated)]
            CollectionDetails::V1 { size } => {
                metadata.collection_details = Some(CollectionDetails::V1 {
                    size: size
                        .checked_add(1)
                        .ok_or(MetadataError::NumericalOverflowError)?,
                });
                msg!("Clean write collection metadata");
                clean_write_metadata(metadata, metadata_info)?;
                Ok(())
            }
        }
    } else {
        msg!("No collection details. Can't increment.");
        Err(MetadataError::UnsizedCollection.into())
    }
}

pub fn decrement_collection_size(
    metadata: &mut Metadata,
    metadata_info: &AccountInfo,
) -> ProgramResult {
    if let Some(ref details) = metadata.collection_details {
        match details {
            #[allow(deprecated)]
            CollectionDetails::V1 { size } => {
                metadata.collection_details = Some(CollectionDetails::V1 {
                    size: size
                        .checked_sub(1)
                        .ok_or(MetadataError::NumericalOverflowError)?,
                });
                clean_write_metadata(metadata, metadata_info)?;
                Ok(())
            }
        }
    } else {
        msg!("No collection details. Can't decrement.");
        Err(MetadataError::UnsizedCollection.into())
    }
}
