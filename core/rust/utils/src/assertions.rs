use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    log::sol_log,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    rent::Rent,
};

pub fn assert_signer(account_info: &AccountInfo) -> ProgramResult {
    if !account_info.is_signer {
        sol_log(
            format!(
                "signer assertion failed for {key}: not signer",
                key = account_info.key,
            )
            .as_str(),
        );
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
        sol_log(
            format!(
                "initialized assertion failed for {key}: not initialized",
                key = account_info.key,
            )
            .as_str(),
        );
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
        sol_log(
            format!(
                "owner assertion failed for {key}: expected {expected}, got {actual}",
                key = account.key,
                expected = owner,
                actual = account.owner
            )
            .as_str(),
        );
        Err(error.into())
    } else {
        Ok(())
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
        sol_log(
            format!(
                "derivation assertion failed for {actual_key}:\n
                \x20   expected {key} with program {program_id}, path {path:?}",
                actual_key = account.key,
                key = key,
                program_id = program_id,
                path = path,
            )
            .as_str(),
        );
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
        sol_log(
            format!(
                "rent exempt assertion failed for {key}: has {balance} lamports, requires at least {min} lamports",
                key = account_info.key,
                balance = account_info.lamports(),
                min = rent.minimum_balance(account_info.data_len()),
            )
            .as_str(),
        );
        Err(error.into())
    } else {
        Ok(())
    }
}
