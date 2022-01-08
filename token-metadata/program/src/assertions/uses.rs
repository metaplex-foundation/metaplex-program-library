use solana_program::program_error::ProgramError;

use crate::{
    error::MetadataError,
    state::{UseMethod, Uses},
};

pub fn assert_no_burn_needed(uses: Option<Uses>) -> Result<(), ProgramError> {
    match uses {
        Some(uses) if uses.use_method == UseMethod::Burn && uses.remaining == 0 => {
            Err(MetadataError::MustBeBurned.into())
        }
        _ => Ok(()),
    }
}
pub fn assert_valid_use(
    incoming_use: &Option<Uses>,
    current_use: &Option<Uses>,
) -> Result<(), ProgramError> {
    let pat = (incoming_use, current_use);
    match pat {
        (Some(i), None) if i.use_method == UseMethod::Single && i.available > 1 => {
            Err(MetadataError::InvalidUseMethod.into())
        }
        (Some(i), None) if i.use_method == UseMethod::Multiple && i.available < 2 => {
            Err(MetadataError::InvalidUseMethod.into())
        }
        _ => Ok(()),
    }
}
