use solana_program::program_error::ProgramError;

use crate::{
    error::MetadataError,
    state::{Collection, DataV2, Metadata},
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
