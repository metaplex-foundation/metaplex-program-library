use borsh::BorshSerialize;
use mpl_utils::assert_signer;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::{
    assertions::{
        assert_owned_by,
        collection::{assert_collection_verify_is_valid, assert_has_collection_authority},
    },
    error::MetadataError,
    instruction::MetadataInstruction,
    state::{Metadata, TokenMetadataAccount},
};

pub(crate) mod instruction {
    use super::*;

    /// # Verify Collection
    ///
    /// If a MetadataAccount Has a Collection allow the UpdateAuthority of the Collection to Verify the NFT Belongs in the Collection
    ///
    /// ### Accounts:
    ///
    ///   0. `[writable]` Metadata account
    ///   1. `[signer]` Collection Update authority
    ///   2. `[signer]` payer
    ///   3. `[]` Mint of the Collection
    ///   4. `[]` Metadata Account of the Collection
    ///   5. `[]` MasterEdition2 Account of the Collection Token
    #[allow(clippy::too_many_arguments)]
    pub fn verify_collection(
        program_id: Pubkey,
        metadata: Pubkey,
        collection_authority: Pubkey,
        payer: Pubkey,
        collection_mint: Pubkey,
        collection: Pubkey,
        collection_master_edition_account: Pubkey,
        collection_authority_record: Option<Pubkey>,
    ) -> Instruction {
        let mut accounts = vec![
            AccountMeta::new(metadata, false),
            AccountMeta::new(collection_authority, true),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(collection_mint, false),
            AccountMeta::new_readonly(collection, false),
            AccountMeta::new_readonly(collection_master_edition_account, false),
        ];

        if let Some(collection_authority_record) = collection_authority_record {
            accounts.push(AccountMeta::new_readonly(
                collection_authority_record,
                false,
            ));
        }

        Instruction {
            program_id,
            accounts,
            data: MetadataInstruction::VerifyCollection.try_to_vec().unwrap(),
        }
    }
}

pub fn verify_collection(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let metadata_info = next_account_info(account_info_iter)?;
    let collection_authority_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let collection_mint = next_account_info(account_info_iter)?;
    let collection_info = next_account_info(account_info_iter)?;
    let edition_account_info = next_account_info(account_info_iter)?;
    let using_delegated_collection_authority = accounts.len() == 7;
    assert_signer(collection_authority_info)?;
    assert_signer(payer_info)?;

    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(collection_info, program_id)?;
    assert_owned_by(collection_mint, &spl_token::id())?;
    assert_owned_by(edition_account_info, program_id)?;

    let mut metadata = Metadata::from_account_info(metadata_info)?;
    let collection_metadata = Metadata::from_account_info(collection_info)?;

    assert_collection_verify_is_valid(
        &metadata.collection,
        &collection_metadata,
        collection_mint,
        edition_account_info,
    )?;

    if using_delegated_collection_authority {
        let collection_authority_record = next_account_info(account_info_iter)?;
        assert_has_collection_authority(
            collection_authority_info,
            &collection_metadata,
            collection_mint.key,
            Some(collection_authority_record),
        )?;
    } else {
        assert_has_collection_authority(
            collection_authority_info,
            &collection_metadata,
            collection_mint.key,
            None,
        )?;
    }

    // This handler can only verify non-sized NFTs
    if collection_metadata.collection_details.is_some() {
        return Err(MetadataError::SizedCollection.into());
    }

    // If the NFT has collection data, we set it to be verified
    if let Some(collection) = &mut metadata.collection {
        collection.verified = true;
        metadata.serialize(&mut *metadata_info.try_borrow_mut_data()?)?;
    }
    Ok(())
}
