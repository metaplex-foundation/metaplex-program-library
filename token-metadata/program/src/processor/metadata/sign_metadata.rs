use borsh::BorshSerialize;
use mpl_utils::assert_signer;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::{
    assertions::assert_owned_by,
    error::MetadataError,
    state::{Metadata, TokenMetadataAccount},
};

pub fn process_sign_metadata(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
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
                creator.verified = true;
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
