use borsh::BorshSerialize;
use mpl_utils::{assert_signer, create_or_allocate_account_raw};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::{
    assertions::{assert_derivation, assert_owned_by},
    error::MetadataError,
    instruction::MetadataInstruction,
    state::{
        CollectionAuthorityRecord, Key, Metadata, TokenMetadataAccount, COLLECTION_AUTHORITY,
        COLLECTION_AUTHORITY_RECORD_SIZE, PREFIX,
    },
};

pub(crate) mod instruction {
    use super::*;

    ///# Approve Collection Authority
    ///
    ///Approve another account to verify NFTs belonging to a collection, [verify_collection] on the collection NFT
    ///
    ///### Accounts:
    ///   0. `[writable]` Collection Authority Record PDA
    ///   1. `[signer]` Update Authority of Collection NFT
    ///   2. `[signer]` Payer
    ///   3. `[]` A Collection Authority
    ///   4. `[]` Collection Metadata account
    ///   5. `[]` Mint of Collection Metadata
    ///   6. `[]` Token program
    ///   7. `[]` System program
    ///   8. Optional `[]` Rent info
    #[allow(clippy::too_many_arguments)]
    pub fn approve_collection_authority(
        program_id: Pubkey,
        collection_authority_record: Pubkey,
        new_collection_authority: Pubkey,
        update_authority: Pubkey,
        payer: Pubkey,
        metadata: Pubkey,
        mint: Pubkey,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(collection_authority_record, false),
                AccountMeta::new_readonly(new_collection_authority, false),
                AccountMeta::new(update_authority, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(metadata, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(solana_program::system_program::id(), false),
            ],
            data: MetadataInstruction::ApproveCollectionAuthority
                .try_to_vec()
                .unwrap(),
        }
    }
}

pub fn approve_collection_authority(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let collection_authority_record = next_account_info(account_info_iter)?;
    let new_collection_authority = next_account_info(account_info_iter)?;
    let update_authority = next_account_info(account_info_iter)?;
    let payer = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;

    let metadata = Metadata::from_account_info(metadata_info)?;
    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(mint_info, &spl_token::id())?;
    assert_signer(update_authority)?;
    assert_signer(payer)?;
    if metadata.update_authority != *update_authority.key {
        return Err(MetadataError::UpdateAuthorityIncorrect.into());
    }
    if metadata.mint != *mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }
    let collection_authority_info_empty = collection_authority_record.try_data_is_empty()?;
    if !collection_authority_info_empty {
        return Err(MetadataError::CollectionAuthorityRecordAlreadyExists.into());
    }
    let collection_authority_path = Vec::from([
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint_info.key.as_ref(),
        COLLECTION_AUTHORITY.as_bytes(),
        new_collection_authority.key.as_ref(),
    ]);
    let collection_authority_bump_seed = &[assert_derivation(
        program_id,
        collection_authority_record,
        &collection_authority_path,
    )?];
    let mut collection_authority_seeds = collection_authority_path.clone();
    collection_authority_seeds.push(collection_authority_bump_seed);
    create_or_allocate_account_raw(
        *program_id,
        collection_authority_record,
        system_account_info,
        payer,
        COLLECTION_AUTHORITY_RECORD_SIZE,
        &collection_authority_seeds,
    )?;

    let mut record = CollectionAuthorityRecord::from_account_info(collection_authority_record)?;
    record.key = Key::CollectionAuthorityRecord;
    record.bump = collection_authority_bump_seed[0];
    record.serialize(&mut *collection_authority_record.try_borrow_mut_data()?)?;
    Ok(())
}
