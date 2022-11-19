use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::{
    assert_signer,
    token::{spl_token_burn, TokenBurnParams},
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::SysvarId,
};

use crate::{
    assertions::{
        assert_owned_by,
        metadata::assert_currently_holding,
        uses::{
            assert_burner, assert_use_authority_derivation, assert_valid_bump,
            process_use_authority_validation,
        },
    },
    error::MetadataError,
    state::{Metadata, TokenMetadataAccount, UseAuthorityRecord, UseMethod, Uses, BURN, PREFIX},
};

pub(crate) mod instruction {
    #[cfg(feature = "serde-feature")]
    use serde::{Deserialize, Serialize};
    use solana_program::instruction::{AccountMeta, Instruction};

    use super::*;
    use crate::instruction::MetadataInstruction;

    #[repr(C)]
    #[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
    #[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
    pub struct UtilizeArgs {
        pub number_of_uses: u64,
    }

    ///# Utilize
    ///
    ///Utilize or Use an NFT , burns the NFT and returns the lamports to the update authority if the use method is burn and its out of uses.
    ///Use Authority can be the Holder of the NFT, or a Delegated Use Authority.
    ///
    ///### Accounts:
    ///
    ///   0. `[writable]` Metadata account
    ///   1. `[writable]` Token Account Of NFT
    ///   2. `[writable]` Mint of the Metadata
    ///   2. `[signer]` A Use Authority / Can be the current Owner of the NFT
    ///   3. `[signer]` Payer
    ///   4. `[]` Owner
    ///   5. `[]` Token program
    ///   6. `[]` Associated Token program
    ///   7. `[]` System program
    ///   8. Optional `[]` Rent info
    ///   9. Optional `[writable]` Use Authority Record PDA If present the program Assumes a delegated use authority
    #[allow(clippy::too_many_arguments)]
    pub fn utilize(
        program_id: Pubkey,
        metadata: Pubkey,
        token_account: Pubkey,
        mint: Pubkey,
        use_authority_record_pda: Option<Pubkey>,
        use_authority: Pubkey,
        owner: Pubkey,
        burner: Option<Pubkey>,
        number_of_uses: u64,
    ) -> Instruction {
        let mut accounts = vec![
            AccountMeta::new(metadata, false),
            AccountMeta::new(token_account, false),
            AccountMeta::new(mint, false),
            AccountMeta::new(use_authority, true),
            AccountMeta::new_readonly(owner, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ];
        if let Some(use_authority_record_pda) = use_authority_record_pda {
            accounts.push(AccountMeta::new(use_authority_record_pda, false));
        }

        if let Some(burner) = burner {
            accounts.push(AccountMeta::new_readonly(burner, false));
        }

        Instruction {
            program_id,
            accounts,
            data: MetadataInstruction::Utilize(UtilizeArgs { number_of_uses })
                .try_to_vec()
                .unwrap(),
        }
    }
}

pub fn utilize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    number_of_uses: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().peekable();
    let metadata_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let user_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let token_program_account_info = next_account_info(account_info_iter)?;
    let _ata_program_account_info = next_account_info(account_info_iter)?;
    let _system_account_info = next_account_info(account_info_iter)?;
    // consume the next account only if it is Rent
    let approved_authority_is_using = if account_info_iter
        .next_if(|info| info.key == &Rent::id())
        .is_some()
    {
        // rent was passed in
        accounts.len() == 11
    } else {
        // necessary accounts is one less if rent isn't passed in.
        accounts.len() == 10
    };

    let metadata: Metadata = Metadata::from_account_info(metadata_info)?;
    if metadata.uses.is_none() {
        return Err(MetadataError::Unusable.into());
    }
    if *token_program_account_info.key != spl_token::id() {
        return Err(MetadataError::InvalidTokenProgram.into());
    }
    assert_signer(user_info)?;
    assert_currently_holding(
        program_id,
        owner_info,
        metadata_info,
        &metadata,
        mint_info,
        token_account_info,
    )?;
    let mut metadata = Metadata::from_account_info(metadata_info)?;
    let metadata_uses = metadata.uses.unwrap();
    let must_burn = metadata_uses.use_method == UseMethod::Burn;
    if number_of_uses > metadata_uses.total || number_of_uses > metadata_uses.remaining {
        return Err(MetadataError::NotEnoughUses.into());
    }
    let remaining_uses = metadata_uses
        .remaining
        .checked_sub(number_of_uses)
        .ok_or(MetadataError::NotEnoughUses)?;
    metadata.uses = Some(Uses {
        use_method: metadata_uses.use_method,
        total: metadata_uses.total,
        remaining: remaining_uses,
    });
    if approved_authority_is_using {
        let use_authority_record_info = next_account_info(account_info_iter)?;
        let data = &mut *use_authority_record_info.try_borrow_mut_data()?;
        process_use_authority_validation(data.len(), false)?;
        assert_owned_by(use_authority_record_info, program_id)?;
        let canonical_bump = assert_use_authority_derivation(
            program_id,
            use_authority_record_info,
            user_info,
            mint_info,
        )?;
        let mut record = UseAuthorityRecord::from_bytes(data)?;
        // Migrates old UARs to having the bump stored
        if record.bump_empty() {
            record.bump = canonical_bump;
        }
        assert_valid_bump(canonical_bump, &record)?;
        record.allowed_uses = record
            .allowed_uses
            .checked_sub(number_of_uses)
            .ok_or(MetadataError::NotEnoughUses)?;
        record.serialize(data)?;
    } else if user_info.key != owner_info.key {
        return Err(MetadataError::InvalidUser.into());
    }
    metadata.serialize(&mut *metadata_info.try_borrow_mut_data()?)?;
    if remaining_uses == 0 && must_burn {
        if approved_authority_is_using {
            let burn_authority_info = next_account_info(account_info_iter)?;
            let seed = assert_burner(burn_authority_info.key)?;
            let burn_bump_ref = &[
                PREFIX.as_bytes(),
                program_id.as_ref(),
                BURN.as_bytes(),
                &[seed],
            ];
            spl_token_burn(TokenBurnParams {
                mint: mint_info.clone(),
                amount: 1,
                authority: burn_authority_info.clone(),
                token_program: token_program_account_info.clone(),
                source: token_account_info.clone(),
                authority_signer_seeds: Some(burn_bump_ref),
            })?;
        } else {
            spl_token_burn(TokenBurnParams {
                mint: mint_info.clone(),
                amount: 1,
                authority: owner_info.clone(),
                token_program: token_program_account_info.clone(),
                source: token_account_info.clone(),
                authority_signer_seeds: None,
            })?;
        }
    }
    Ok(())
}
