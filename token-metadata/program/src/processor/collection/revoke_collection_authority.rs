use borsh::BorshSerialize;
use mpl_utils::assert_signer;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program_memory::sol_memset,
    pubkey::Pubkey,
};

use crate::{
    assertions::{assert_owned_by, collection::assert_has_collection_authority},
    error::MetadataError,
    instruction::MetadataInstruction,
    state::{Metadata, TokenMetadataAccount, USE_AUTHORITY_RECORD_SIZE},
};

pub(crate) mod instruction {
    use super::*;

    //# Revoke Collection Authority
    ///
    ///Revoke account to call [verify_collection] on this NFT
    ///
    ///### Accounts:
    ///
    ///   0. `[writable]` Collection Authority Record PDA
    ///   1. `[writable]` The Authority that was delegated to
    ///   2. `[signer]` The Original Update Authority or Delegated Authority
    ///   2. `[]` Metadata account
    ///   3. `[]` Mint of Metadata
    #[allow(clippy::too_many_arguments)]
    pub fn revoke_collection_authority(
        program_id: Pubkey,
        collection_authority_record: Pubkey,
        delegate_authority: Pubkey,
        revoke_authority: Pubkey,
        metadata: Pubkey,
        mint: Pubkey,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(collection_authority_record, false),
                AccountMeta::new_readonly(delegate_authority, false),
                AccountMeta::new(revoke_authority, true),
                AccountMeta::new_readonly(metadata, false),
                AccountMeta::new_readonly(mint, false),
            ],
            data: MetadataInstruction::RevokeCollectionAuthority
                .try_to_vec()
                .unwrap(),
        }
    }
}

pub fn revoke_collection_authority(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let collection_authority_record = next_account_info(account_info_iter)?;
    let delegate_authority = next_account_info(account_info_iter)?;
    let revoke_authority = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let metadata = Metadata::from_account_info(metadata_info)?;

    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(mint_info, &spl_token::id())?;
    assert_signer(revoke_authority)?;

    if metadata.update_authority != *revoke_authority.key
        && *delegate_authority.key != *revoke_authority.key
    {
        return Err(MetadataError::RevokeCollectionAuthoritySignerIncorrect.into());
    }
    if metadata.mint != *mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    let collection_authority_info_empty = collection_authority_record.try_data_is_empty()?;
    if collection_authority_info_empty {
        return Err(MetadataError::CollectionAuthorityDoesNotExist.into());
    }
    assert_has_collection_authority(
        delegate_authority,
        &metadata,
        mint_info.key,
        Some(collection_authority_record),
    )?;
    let lamports = collection_authority_record.lamports();
    **collection_authority_record.try_borrow_mut_lamports()? = 0;
    **revoke_authority.try_borrow_mut_lamports()? = revoke_authority
        .lamports()
        .checked_add(lamports)
        .ok_or(MetadataError::NumericalOverflowError)?;
    sol_memset(
        *collection_authority_record.try_borrow_mut_data()?,
        0,
        USE_AUTHORITY_RECORD_SIZE,
    );

    Ok(())
}
