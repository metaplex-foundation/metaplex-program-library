use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke, rent::Rent,
    sysvar::Sysvar,
};

use crate::state::{fee::CREATE_FEE, Metadata, TokenMetadataAccount, METADATA_FEE_FLAG_INDEX};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct LevyArgs<'a> {
    pub payer_account_info: &'a AccountInfo<'a>,
    pub token_metadata_pda_info: &'a AccountInfo<'a>,
}

pub(crate) fn levy(args: LevyArgs) -> ProgramResult {
    // Fund metadata account with rent + Metaplex fee.
    let rent = Rent::get()?;

    let fee = CREATE_FEE + rent.minimum_balance(Metadata::size());

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

    Ok(())
}

pub(crate) fn set_fee_flag(pda_account_info: &AccountInfo) -> ProgramResult {
    let mut data = pda_account_info.try_borrow_mut_data()?;
    data[METADATA_FEE_FLAG_INDEX] = 1;

    Ok(())
}

pub(crate) fn clear_fee_flag(pda_account_info: &AccountInfo) -> ProgramResult {
    let mut data = pda_account_info.try_borrow_mut_data()?;

    // Clear the flag if the index exists.
    if let Some(flag) = data.get_mut(METADATA_FEE_FLAG_INDEX) {
        *flag = 0;
    }

    Ok(())
}
