use borsh::BorshSerialize;
pub use instruction::*;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::{
    assertions::assert_owned_by,
    instruction::MetadataInstruction,
    state::{Metadata, TokenMetadataAccount, EDITION, PREFIX},
    utils::puff_out_data_fields,
};

mod instruction {
    use super::*;

    /// puff metadata account instruction
    pub fn puff_metadata_account(program_id: Pubkey, metadata_account: Pubkey) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![AccountMeta::new(metadata_account, false)],
            data: MetadataInstruction::PuffMetadata.try_to_vec().unwrap(),
        }
    }
}

/// Puff out the variable length fields to a fixed length on a metadata
/// account in a permissionless way.
pub fn process_puff_metadata_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let metadata_account_info = next_account_info(account_info_iter)?;
    let mut metadata = Metadata::from_account_info(metadata_account_info)?;

    assert_owned_by(metadata_account_info, program_id)?;

    puff_out_data_fields(&mut metadata);

    let edition_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        metadata.mint.as_ref(),
        EDITION.as_bytes(),
    ];
    let (_, edition_bump_seed) = Pubkey::find_program_address(edition_seeds, program_id);
    metadata.edition_nonce = Some(edition_bump_seed);

    metadata.serialize(&mut *metadata_account_info.try_borrow_mut_data()?)?;
    Ok(())
}
