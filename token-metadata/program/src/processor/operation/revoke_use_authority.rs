use borsh::BorshSerialize;
use mpl_utils::assert_signer;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::invoke,
    program_memory::sol_memset,
    pubkey::Pubkey,
};
use spl_token::instruction::revoke;

use crate::{
    assertions::{
        assert_owned_by,
        metadata::assert_currently_holding,
        uses::{
            assert_use_authority_derivation, assert_valid_bump, process_use_authority_validation,
        },
    },
    error::MetadataError,
    instruction::MetadataInstruction,
    state::{
        Metadata, TokenMetadataAccount, UseAuthorityRecord, UseMethod, USE_AUTHORITY_RECORD_SIZE,
    },
};

pub(crate) mod instruction {
    use super::*;

    //# Revoke Use Authority
    ///
    ///Revoke account to call [utilize] on this NFT
    ///
    ///### Accounts:
    ///
    ///   0. `[writable]` Use Authority Record PDA
    ///   1. `[writable]` Owned Token Account Of Mint
    ///   2. `[signer]` Owner
    ///   3. `[signer]` Payer
    ///   4. `[]` A Use Authority
    ///   5. `[]` Metadata account
    ///   6. `[]` Mint of Metadata
    ///   7. `[]` Token program
    ///   8. `[]` System program
    ///   9. Optional `[]` Rent info
    #[allow(clippy::too_many_arguments)]
    pub fn revoke_use_authority(
        program_id: Pubkey,
        use_authority_record: Pubkey,
        user: Pubkey,
        owner: Pubkey,
        owner_token_account: Pubkey,
        metadata: Pubkey,
        mint: Pubkey,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(use_authority_record, false),
                AccountMeta::new(owner, true),
                AccountMeta::new_readonly(user, false),
                AccountMeta::new(owner_token_account, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(metadata, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(solana_program::system_program::id(), false),
            ],
            data: MetadataInstruction::RevokeUseAuthority
                .try_to_vec()
                .unwrap(),
        }
    }
}

pub fn revoke_use_authority(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let use_authority_record_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let user_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let token_program_account_info = next_account_info(account_info_iter)?;
    let metadata = Metadata::from_account_info(metadata_info)?;
    if metadata.uses.is_none() {
        return Err(MetadataError::Unusable.into());
    }
    if *token_program_account_info.key != spl_token::id() {
        return Err(MetadataError::InvalidTokenProgram.into());
    }
    assert_signer(owner_info)?;
    assert_currently_holding(
        program_id,
        owner_info,
        metadata_info,
        &metadata,
        mint_info,
        token_account_info,
    )?;
    let data = &mut use_authority_record_info.try_borrow_mut_data()?;
    process_use_authority_validation(data.len(), false)?;
    assert_owned_by(use_authority_record_info, program_id)?;
    let canonical_bump = assert_use_authority_derivation(
        program_id,
        use_authority_record_info,
        user_info,
        mint_info,
    )?;
    let mut record = UseAuthorityRecord::from_bytes(data)?;
    if record.bump_empty() {
        record.bump = canonical_bump;
    }
    assert_valid_bump(canonical_bump, &record)?;
    let metadata_uses = metadata.uses.unwrap();
    if metadata_uses.use_method == UseMethod::Burn {
        invoke(
            &revoke(
                token_program_account_info.key,
                token_account_info.key,
                owner_info.key,
                &[],
            )
            .unwrap(),
            &[
                token_program_account_info.clone(),
                token_account_info.clone(),
                owner_info.clone(),
            ],
        )?;
    }
    let lamports = use_authority_record_info.lamports();
    **use_authority_record_info.try_borrow_mut_lamports()? = 0;
    **owner_info.try_borrow_mut_lamports()? = owner_info
        .lamports()
        .checked_add(lamports)
        .ok_or(MetadataError::NumericalOverflowError)?;
    sol_memset(data, 0, USE_AUTHORITY_RECORD_SIZE);
    Ok(())
}
