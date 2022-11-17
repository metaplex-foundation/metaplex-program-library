use borsh::BorshSerialize;
pub use instruction::*;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::state::Account;

use crate::{
    assertions::{assert_initialized, assert_owned_by},
    error::MetadataError,
    instruction::MetadataInstruction,
    state::{Metadata, TokenMetadataAccount},
};

mod instruction {
    use super::*;

    /// creates a update_primary_sale_happened_via_token instruction
    #[allow(clippy::too_many_arguments)]
    pub fn update_primary_sale_happened_via_token(
        program_id: Pubkey,
        metadata: Pubkey,
        owner: Pubkey,
        token: Pubkey,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(metadata, false),
                AccountMeta::new_readonly(owner, true),
                AccountMeta::new_readonly(token, false),
            ],
            data: MetadataInstruction::UpdatePrimarySaleHappenedViaToken
                .try_to_vec()
                .unwrap(),
        }
    }
}

pub fn process_update_primary_sale_happened_via_token(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let metadata_account_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;

    let token_account: Account = assert_initialized(token_account_info)?;
    let mut metadata = Metadata::from_account_info(metadata_account_info)?;

    assert_owned_by(metadata_account_info, program_id)?;
    assert_owned_by(token_account_info, &spl_token::id())?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if token_account.owner != *owner_info.key {
        return Err(MetadataError::OwnerMismatch.into());
    }

    if token_account.amount == 0 {
        return Err(MetadataError::NoBalanceInAccountForAuthorization.into());
    }

    if token_account.mint != metadata.mint {
        return Err(MetadataError::MintMismatch.into());
    }

    metadata.primary_sale_happened = true;
    metadata.serialize(&mut *metadata_account_info.try_borrow_mut_data()?)?;

    Ok(())
}
