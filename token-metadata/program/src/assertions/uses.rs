use solana_program::program_error::ProgramError;

use crate::{
    error::MetadataError,
    state::{UseMethod, Uses},
};

pub fn assert_valid_use(
    incoming_use: &Option<Uses>,
    current_use: &Option<Uses>,
) -> Result<(), ProgramError> {
    if let Some(i) = incoming_use {
        if i.use_method == UseMethod::Single && (i.total != 1 || i.remaining != 1) {
            return Err(MetadataError::InvalidUseMethod.into());
        }
        if i.use_method == UseMethod::Multiple && i.total < 2 {
            return Err(MetadataError::InvalidUseMethod.into());
        }
    }
    return match (incoming_use, current_use) {
        (Some(incoming), Some(current)) => {
            if incoming.use_method != current.use_method && current.total != current.remaining {
                return Err(MetadataError::CannotChangeUseMethodAfterFirstUse.into());
            }
            if incoming.total != current.total && current.total != current.remaining {
                return Err(MetadataError::CannotChangeUsesAfterFirstUse.into());
            }
            if incoming.remaining != current.remaining && current.total != current.remaining {
                return Err(MetadataError::CannotChangeUsesAfterFirstUse.into());
            }
            Ok(())
        }
        _ => Ok(()),
    };
}
