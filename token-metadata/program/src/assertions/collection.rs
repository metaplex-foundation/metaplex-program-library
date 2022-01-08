use solana_program::program_error::ProgramError;

use crate::{
    error::MetadataError,
    state::{Collection, DataV2, Metadata},
};

pub fn assert_collection_update_is_valid(
    existing: &Option<Collection>,
    incoming: &Option<Collection>,
) -> Result<(), ProgramError> {
    if !existing.is_some() && incoming.is_some() && incoming.as_ref().unwrap().verified == true {
        return Err(MetadataError::CollectionCannotBeVerifiedInThisInstruction.into());
    }
    Ok(())
}
