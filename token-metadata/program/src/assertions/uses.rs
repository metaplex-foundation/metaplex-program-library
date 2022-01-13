use solana_program::{program_error::ProgramError, account_info::AccountInfo, pubkey::Pubkey};

use crate::{
    error::MetadataError,
    state::{UseMethod, Uses, USER, PREFIX}, utils::assert_derivation,
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

pub fn process_use_authority_validation(
    program_id: &Pubkey,
    use_authority_record_info: &AccountInfo,
    user_info: &AccountInfo,
    mint_info: &AccountInfo,
    must_be_empty: bool,
) -> Result<u8, ProgramError> {
    let record_info_empty = use_authority_record_info.try_data_is_empty()?;
    if must_be_empty {
        if !record_info_empty {
            return Err(MetadataError::UseAuthorityRecordAlreadyExists.into());
        }
    } else {
        if record_info_empty {
            return Err(MetadataError::UseAuthorityRecordAlreadyRevoked.into());
        }
    }
    let use_authority_seeds = [
        PREFIX.as_bytes(),
        program_id.as_ref(),
        &mint_info.key.as_ref(),
        USER.as_bytes(),
        &user_info.key.as_ref(),
    ];

    assert_derivation(&program_id, use_authority_record_info, &use_authority_seeds)
}
