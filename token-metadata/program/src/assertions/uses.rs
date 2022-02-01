use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::MetadataError,
    state::{UseAuthorityRecord, UseMethod, Uses, PREFIX, USER},
    utils::assert_derivation,
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
    let data = use_authority_record_info.try_borrow_data()?;
    let record_info_empty = data.len() == 0;
    let use_authority_seeds = [
        PREFIX.as_bytes(),
        program_id.as_ref(),
        &mint_info.key.as_ref(),
        USER.as_bytes(),
        &user_info.key.as_ref(),
    ];
    let derivation =
        assert_derivation(&program_id, use_authority_record_info, &use_authority_seeds)?;
    if must_be_empty {
        if !record_info_empty {
            return Err(MetadataError::UseAuthorityRecordAlreadyExists.into());
        }
    } else {
        if record_info_empty {
            return Err(MetadataError::UseAuthorityRecordAlreadyRevoked.into());
        }
        let record_data = UseAuthorityRecord::from_bytes(&data)?;
        if derivation != record_data.bump {
            return Err(MetadataError::InvalidUseAuthorityRecord.into());
        }
    }
    Ok(derivation)
}
