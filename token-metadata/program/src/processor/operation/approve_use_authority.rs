use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::{assert_signer, create_or_allocate_account_raw};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::invoke,
    pubkey::Pubkey,
};
use spl_token::instruction::approve;

use crate::{
    assertions::{
        metadata::assert_currently_holding,
        uses::{assert_burner, assert_use_authority_derivation, process_use_authority_validation},
    },
    error::MetadataError,
    instruction::MetadataInstruction,
    state::{
        Key, Metadata, TokenMetadataAccount, UseAuthorityRecord, UseMethod, PREFIX, USER,
        USE_AUTHORITY_RECORD_SIZE,
    },
};

pub(crate) mod instruction {
    #[cfg(feature = "serde-feature")]
    use serde::{Deserialize, Serialize};

    use super::*;

    #[repr(C)]
    #[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
    #[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
    pub struct ApproveUseAuthorityArgs {
        pub number_of_uses: u64,
    }

    ///# Approve Use Authority
    ///
    ///Approve another account to call [utilize] on this NFT
    ///
    ///### Args:
    ///
    ///See: [ApproveUseAuthorityArgs]
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
    ///   7. `[]` Program As Signer (Burner)
    ///   8. `[]` Token program
    ///   9. `[]` System program
    ///   10. Optional `[]` Rent info
    #[allow(clippy::too_many_arguments)]
    pub fn approve_use_authority(
        program_id: Pubkey,
        use_authority_record: Pubkey,
        user: Pubkey,
        owner: Pubkey,
        payer: Pubkey,
        owner_token_account: Pubkey,
        metadata: Pubkey,
        mint: Pubkey,
        burner: Pubkey,
        number_of_uses: u64,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(use_authority_record, false),
                AccountMeta::new(owner, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(user, false),
                AccountMeta::new(owner_token_account, false),
                AccountMeta::new_readonly(metadata, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(burner, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(solana_program::system_program::id(), false),
            ],
            data: MetadataInstruction::ApproveUseAuthority(ApproveUseAuthorityArgs {
                number_of_uses,
            })
            .try_to_vec()
            .unwrap(),
        }
    }
}

pub fn approve_use_authority(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    number_of_uses: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let use_authority_record_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let payer = next_account_info(account_info_iter)?;
    let user_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let program_as_burner = next_account_info(account_info_iter)?;
    let token_program_account_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let metadata: Metadata = Metadata::from_account_info(metadata_info)?;

    if metadata.uses.is_none() {
        return Err(MetadataError::Unusable.into());
    }
    if *token_program_account_info.key != spl_token::id() {
        return Err(MetadataError::InvalidTokenProgram.into());
    }
    assert_signer(owner_info)?;
    assert_signer(payer)?;
    assert_currently_holding(
        program_id,
        owner_info,
        metadata_info,
        &metadata,
        mint_info,
        token_account_info,
    )?;
    let metadata_uses = metadata.uses.unwrap();
    let bump_seed = assert_use_authority_derivation(
        program_id,
        use_authority_record_info,
        user_info,
        mint_info,
    )?;
    let use_authority_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint_info.key.as_ref(),
        USER.as_bytes(),
        user_info.key.as_ref(),
        &[bump_seed],
    ];
    process_use_authority_validation(use_authority_record_info.data_len(), true)?;
    create_or_allocate_account_raw(
        *program_id,
        use_authority_record_info,
        system_account_info,
        payer,
        USE_AUTHORITY_RECORD_SIZE,
        use_authority_seeds,
    )?;
    if number_of_uses > metadata_uses.remaining {
        return Err(MetadataError::NotEnoughUses.into());
    }
    if metadata_uses.use_method == UseMethod::Burn {
        assert_burner(program_as_burner.key)?;
        invoke(
            &approve(
                token_program_account_info.key,
                token_account_info.key,
                program_as_burner.key,
                owner_info.key,
                &[],
                1,
            )
            .unwrap(),
            &[
                token_program_account_info.clone(),
                token_account_info.clone(),
                program_as_burner.clone(),
                owner_info.clone(),
            ],
        )?;
    }
    let mutable_data = &mut *use_authority_record_info.try_borrow_mut_data()?;
    let mut record = UseAuthorityRecord::from_bytes(*mutable_data)?;
    record.key = Key::UseAuthorityRecord;
    record.allowed_uses = number_of_uses;
    record.bump = bump_seed;
    record.serialize(mutable_data)?;
    Ok(())
}
