use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    rent::Rent,
};

use crate::cmp_pubkeys;

pub fn assert_signer(account_info: &AccountInfo) -> ProgramResult {
    if !account_info.is_signer {
        Err(ProgramError::MissingRequiredSignature)
    } else {
        Ok(())
    }
}

pub fn assert_initialized<T: Pack + IsInitialized>(
    account_info: &AccountInfo,
    error: impl Into<ProgramError>,
) -> Result<T, ProgramError> {
    let account: T = T::unpack_unchecked(&account_info.data.borrow())?;
    if !account.is_initialized() {
        Err(error.into())
    } else {
        Ok(account)
    }
}

pub fn assert_owned_by(
    account: &AccountInfo,
    owner: &Pubkey,
    error: impl Into<ProgramError>,
) -> ProgramResult {
    if account.owner != owner {
        Err(error.into())
    } else {
        Ok(())
    }
}

pub fn assert_owner_in(
    account: &AccountInfo,
    owners: &[Pubkey],
    error: impl Into<ProgramError>,
) -> ProgramResult {
    if owners.iter().any(|owner| cmp_pubkeys(owner, account.owner)) {
        Ok(())
    } else {
        Err(error.into())
    }
}

pub fn assert_derivation(
    program_id: &Pubkey,
    account: &AccountInfo,
    path: &[&[u8]],
    error: impl Into<ProgramError>,
) -> Result<u8, ProgramError> {
    let (key, bump) = Pubkey::find_program_address(path, program_id);
    if key != *account.key {
        return Err(error.into());
    }
    Ok(bump)
}

pub fn assert_rent_exempt(
    rent: &Rent,
    account_info: &AccountInfo,
    error: impl Into<ProgramError>,
) -> ProgramResult {
    if !rent.is_exempt(account_info.lamports(), account_info.data_len()) {
        Err(error.into())
    } else {
        Ok(())
    }
}
