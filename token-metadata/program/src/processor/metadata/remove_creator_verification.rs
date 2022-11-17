use borsh::BorshSerialize;
use mpl_utils::assert_signer;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::{
    assertions::assert_owned_by,
    error::MetadataError,
    instruction_old::MetadataInstruction,
    state::{Metadata, TokenMetadataAccount},
};

pub(crate) mod instruction {
    use super::*;

    /// Remove Creator Verificaton
    #[allow(clippy::too_many_arguments)]
    pub fn remove_creator_verification(
        program_id: Pubkey,
        metadata: Pubkey,
        creator: Pubkey,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(metadata, false),
                AccountMeta::new_readonly(creator, true),
            ],
            data: MetadataInstruction::RemoveCreatorVerification
                .try_to_vec()
                .unwrap(),
        }
    }
}

pub fn process_remove_creator_verification(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let metadata_info = next_account_info(account_info_iter)?;
    let creator_info = next_account_info(account_info_iter)?;

    assert_signer(creator_info)?;
    assert_owned_by(metadata_info, program_id)?;

    let mut metadata = Metadata::from_account_info(metadata_info)?;

    if let Some(creators) = &mut metadata.data.creators {
        let mut found = false;
        for creator in creators {
            if creator.address == *creator_info.key {
                creator.verified = false;
                found = true;
                break;
            }
        }
        if !found {
            return Err(MetadataError::CreatorNotFound.into());
        }
    } else {
        return Err(MetadataError::NoCreatorsPresentOnMetadata.into());
    }
    metadata.serialize(&mut *metadata_info.try_borrow_mut_data()?)?;

    Ok(())
}
