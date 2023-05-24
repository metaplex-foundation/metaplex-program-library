use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke, rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    error::MetadataError,
    state::{
        fee::{CREATE_FEE, UPDATE_FEE},
        Key, Metadata, TokenMetadataAccount, MASTER_EDITION_FEE_FLAG_INDEX,
        METADATA_FEE_FLAG_INDEX,
    },
};

#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum IxType {
    CreateMetadata,
    UpdateMetadata,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct LevyArgs<'a> {
    pub ix_type: IxType,
    pub payer_account_info: &'a AccountInfo<'a>,
    pub token_metadata_pda_info: &'a AccountInfo<'a>,
    pub include_rent: bool,
}

pub(crate) fn levy(args: LevyArgs) -> ProgramResult {
    // Fund metadata account with rent + Metaplex fee.
    let rent = Rent::get()?;

    let fee = match args.ix_type {
        IxType::CreateMetadata => {
            if args.include_rent {
                CREATE_FEE + rent.minimum_balance(Metadata::size())
            } else {
                CREATE_FEE
            }
        }
        IxType::UpdateMetadata => UPDATE_FEE,
    };

    invoke(
        &solana_program::system_instruction::transfer(
            args.payer_account_info.key,
            args.token_metadata_pda_info.key,
            fee,
        ),
        &[
            args.payer_account_info.clone(),
            args.token_metadata_pda_info.clone(),
        ],
    )?;

    // Create ixes must set the fee flag after the metadata account is populated, but
    // levy() must be called before to fund the account with rent + fee.
    if args.ix_type != IxType::CreateMetadata {
        set_fee_flag(args.token_metadata_pda_info, args.ix_type)?;
    }

    Ok(())
}

pub(crate) fn set_fee_flag(pda_account_info: &AccountInfo, ix_type: IxType) -> ProgramResult {
    let flags_index = match ix_type {
        IxType::CreateMetadata | IxType::UpdateMetadata => METADATA_FEE_FLAG_INDEX,
    };

    let mut data = pda_account_info.try_borrow_mut_data()?;
    data[flags_index] = 1;

    Ok(())
}

pub(crate) fn clear_fee_flag(pda_account_info: &AccountInfo, key: Key) -> ProgramResult {
    let flags_index = match key {
        Key::Uninitialized | Key::MetadataV1 => METADATA_FEE_FLAG_INDEX,
        Key::MasterEditionV2 => MASTER_EDITION_FEE_FLAG_INDEX,
        _ => return Err(MetadataError::InvalidMetadataKey.into()),
    };

    let mut data = pda_account_info.try_borrow_mut_data()?;
    // Clear the flag if the index exists.
    if let Some(flag) = data.get_mut(flags_index) {
        *flag = 0;
    }

    Ok(())
}
