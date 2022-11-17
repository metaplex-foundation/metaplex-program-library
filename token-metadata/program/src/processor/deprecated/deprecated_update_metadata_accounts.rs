use borsh::{BorshDeserialize, BorshSerialize};
pub use instruction::*;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::{
    assertions::{
        assert_owned_by,
        metadata::{assert_data_valid, assert_update_authority_is_correct},
    },
    error::MetadataError,
    instruction::MetadataInstruction,
    state::{Data, Metadata, TokenMetadataAccount},
    utils::puff_out_data_fields,
};

mod instruction {
    #[cfg(feature = "serde-feature")]
    use {
        serde::{Deserialize, Serialize},
        serde_with::{As, DisplayFromStr},
    };

    use super::*;

    #[repr(C)]
    #[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
    #[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
    /// Args for update call
    pub struct UpdateMetadataAccountArgs {
        pub data: Option<Data>,
        #[cfg_attr(
            feature = "serde-feature",
            serde(with = "As::<Option<DisplayFromStr>>")
        )]
        pub update_authority: Option<Pubkey>,
        pub primary_sale_happened: Option<bool>,
    }

    /// update metadata account instruction
    /// #[deprecated(since="1.1.0", note="please use `update_metadata_accounts_v2` instead")]
    pub fn update_metadata_accounts(
        program_id: Pubkey,
        metadata_account: Pubkey,
        update_authority: Pubkey,
        new_update_authority: Option<Pubkey>,
        data: Option<Data>,
        primary_sale_happened: Option<bool>,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(metadata_account, false),
                AccountMeta::new_readonly(update_authority, true),
            ],
            data: MetadataInstruction::UpdateMetadataAccount(UpdateMetadataAccountArgs {
                data,
                update_authority: new_update_authority,
                primary_sale_happened,
            })
            .try_to_vec()
            .unwrap(),
        }
    }
}

/// Update existing account instruction
pub fn process_deprecated_update_metadata_accounts(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    optional_data: Option<Data>,
    update_authority: Option<Pubkey>,
    primary_sale_happened: Option<bool>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let metadata_account_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let mut metadata = Metadata::from_account_info(metadata_account_info)?;

    assert_owned_by(metadata_account_info, program_id)?;
    assert_update_authority_is_correct(&metadata, update_authority_info)?;

    if let Some(data) = optional_data {
        if metadata.is_mutable {
            assert_data_valid(
                &data,
                update_authority_info.key,
                &metadata,
                false,
                update_authority_info.is_signer,
            )?;
            metadata.data = data;
        } else {
            return Err(MetadataError::DataIsImmutable.into());
        }
    }

    if let Some(val) = update_authority {
        metadata.update_authority = val;
    }

    if let Some(val) = primary_sale_happened {
        if val {
            metadata.primary_sale_happened = val
        } else {
            return Err(MetadataError::PrimarySaleCanOnlyBeFlippedToTrue.into());
        }
    }

    puff_out_data_fields(&mut metadata);

    metadata.serialize(&mut *metadata_account_info.data.borrow_mut())?;
    Ok(())
}
