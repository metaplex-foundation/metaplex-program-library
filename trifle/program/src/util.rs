use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke, rent::Rent,
    system_instruction, sysvar::Sysvar,
};

use crate::{
    error::TrifleError,
    state::escrow_constraints::{fees, EscrowConstraintModel, RoyaltyInstruction},
};

/// Resize an account using realloc, lifted from Solana Cookbook
#[inline(always)]
pub fn resize_or_reallocate_account_raw<'a>(
    target_account: &AccountInfo<'a>,
    funding_account: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    new_size: usize,
) -> ProgramResult {
    let rent = Rent::get()?;
    let new_minimum_balance = rent.minimum_balance(new_size);

    let lamports_diff = new_minimum_balance.saturating_sub(target_account.lamports());
    invoke(
        &system_instruction::transfer(funding_account.key, target_account.key, lamports_diff),
        &[
            funding_account.clone(),
            target_account.clone(),
            system_program.clone(),
        ],
    )?;

    target_account.realloc(new_size, false)?;

    Ok(())
}

pub fn pay_royalties<'a>(
    instruction: RoyaltyInstruction,
    model: &mut EscrowConstraintModel,
    payer: &AccountInfo<'a>,
    royalty_recipient: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let royalty = *model.royalties.get(&instruction).unwrap_or(&0);
    solana_program::msg!("Here");
    invoke(
        &system_instruction::transfer(
            payer.key,
            royalty_recipient.key,
            royalty + fees().get(&instruction).unwrap_or(&0),
        ),
        &[
            payer.clone(),
            royalty_recipient.clone(),
            system_program.clone(),
        ],
    )?;
    solana_program::msg!("Here also");

    model.royalty_balance += royalty
        .checked_mul(8)
        .ok_or(TrifleError::NumericalOverflow)?
        .checked_div(10)
        .ok_or(TrifleError::NumericalOverflow)?;

    Ok(())
}

pub fn is_creation_instruction(hash: u8) -> bool {
    // Check for matches against Create Constraint Model or any of the Add Constraint instructions.
    matches!(hash, 0 | 4 | 5 | 6)
}
