use bitflags::bitflags;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke, rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    error::MetadataError,
    state::{
        Key, Metadata, TokenMetadataAccount, MASTER_EDITION_FEE_FLAG_INDEX, METADATA_FLAGS_INDEX,
    },
};

#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};

// base fee level, 0.001 SOL
pub const BASE_FEE: u64 = 1_000_000;

// create_metadata_accounts
pub const CREATE_FEE: u64 = 10 * BASE_FEE;
pub const UPDATE_FEE: u64 = 2 * BASE_FEE;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct MetadataFlags: u8 {
        const FEES = 0b00000001;
    }
}

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
}

pub(crate) fn levy(args: LevyArgs) -> ProgramResult {
    // Fund metadata account with rent + Metaplex fee.
    let rent = Rent::get()?;

    let fee = match args.ix_type {
        IxType::CreateMetadata => CREATE_FEE + rent.minimum_balance(Metadata::size()),
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
        IxType::CreateMetadata | IxType::UpdateMetadata => METADATA_FLAGS_INDEX,
    };

    let mut data = pda_account_info.try_borrow_mut_data()?;
    let flags_bits = data
        .get(flags_index)
        .ok_or(MetadataError::InvalidMetadataFlags)?;
    let mut flags =
        MetadataFlags::from_bits(*flags_bits).ok_or(MetadataError::InvalidMetadataFlags)?;

    flags.set(MetadataFlags::FEES, true);
    data[flags_index] = flags.bits();

    Ok(())
}

pub(crate) fn clear_fee_flag(pda_account_info: &AccountInfo, key: Key) -> ProgramResult {
    let flags_index = match key {
        Key::Uninitialized | Key::MetadataV1 => METADATA_FLAGS_INDEX,
        Key::MasterEditionV2 => MASTER_EDITION_FEE_FLAG_INDEX,
        _ => return Err(MetadataError::InvalidMetadataKey.into()),
    };

    let mut data = pda_account_info.try_borrow_mut_data()?;
    let flags_bits = data
        .get(flags_index)
        .ok_or(MetadataError::InvalidMetadataFlags)?;
    let mut flags =
        MetadataFlags::from_bits(*flags_bits).ok_or(MetadataError::InvalidMetadataFlags)?;

    flags.set(MetadataFlags::FEES, false);
    data[flags_index] = flags.bits();

    Ok(())
}
