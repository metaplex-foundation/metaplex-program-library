use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction, system_program,
    sysvar::Sysvar,
};

/// Create account almost from scratch, lifted from
/// <https://github.com/solana-labs/solana-program-library/tree/master/associated-token-account/program/src/processor.rs#L51-L98>
pub fn create_or_allocate_account_raw<'a>(
    program_id: Pubkey,
    new_account_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    size: usize,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    let rent = &Rent::get()?;
    let required_lamports = rent
        .minimum_balance(size)
        .max(1)
        .saturating_sub(new_account_info.lamports());

    if required_lamports > 0 {
        msg!("Transfer {} lamports to the new account", required_lamports);
        invoke(
            &system_instruction::transfer(payer_info.key, new_account_info.key, required_lamports),
            &[
                payer_info.clone(),
                new_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;
    }

    let accounts = &[new_account_info.clone(), system_program_info.clone()];

    msg!("Allocate space for the account");
    invoke_signed(
        &system_instruction::allocate(new_account_info.key, size.try_into().unwrap()),
        accounts,
        &[signer_seeds],
    )?;

    msg!("Assign the account to the owning program");
    invoke_signed(
        &system_instruction::assign(new_account_info.key, &program_id),
        accounts,
        &[signer_seeds],
    )?;

    Ok(())
}

/// Resize an account using realloc, lifted from Solana Cookbook
pub fn resize_or_reallocate_account_raw<'a>(
    target_account: &AccountInfo<'a>,
    funding_account: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    new_size: usize,
) -> ProgramResult {
    let rent = Rent::get()?;
    let new_minimum_balance = rent.minimum_balance(new_size);
    let lamports_diff = new_minimum_balance.abs_diff(target_account.lamports());
    if new_size == target_account.data_len() {
        return Ok(());
    }

    let account_infos = &[
        funding_account.clone(),
        target_account.clone(),
        system_program.clone(),
    ];

    if new_size > target_account.data_len() {
        invoke(
            &system_instruction::transfer(funding_account.key, target_account.key, lamports_diff),
            account_infos,
        )?;
    } else if target_account.owner == system_program.key {
        invoke(
            &system_instruction::transfer(target_account.key, funding_account.key, lamports_diff),
            account_infos,
        )?;
    } else {
        (**target_account.try_borrow_mut_lamports()?)
            .checked_sub(lamports_diff)
            .ok_or(ProgramError::InvalidRealloc)?;
        (**funding_account.try_borrow_mut_lamports()?)
            .checked_add(lamports_diff)
            .ok_or(ProgramError::InvalidRealloc)?;
    }

    target_account.realloc(new_size, false)
}

/// Close src_account and transfer lamports to dst_account, lifted from Solana Cookbook
pub fn close_account_raw<'a>(
    dest_account_info: &AccountInfo<'a>,
    src_account_info: &AccountInfo<'a>,
) -> ProgramResult {
    let dest_starting_lamports = dest_account_info.lamports();
    let mut dest_lamports_mut = dest_account_info
        .lamports
        .try_borrow_mut()
        .map_err(|_| ProgramError::AccountBorrowFailed)?;
    **dest_lamports_mut = dest_starting_lamports
        .checked_add(src_account_info.lamports())
        .ok_or(ProgramError::InvalidRealloc)?;

    let mut src_lamports_mut = src_account_info
        .lamports
        .try_borrow_mut()
        .map_err(|_| ProgramError::AccountBorrowFailed)?;
    **src_lamports_mut = 0;

    src_account_info.assign(&system_program::ID);
    src_account_info.realloc(0, false).map_err(Into::into)
}
