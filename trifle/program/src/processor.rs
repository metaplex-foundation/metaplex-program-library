use crate::{
    error::TrifleError,
    instruction::{
        AddConstraintToEscrowConstraintModelArgs, CreateEscrowConstraintModelAccountArgs,
        TrifleInstruction,
    },
    state::{EscrowConstraintModel, Key, ESCROW_PREFIX},
};
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_metadata::utils::{
    assert_derivation, assert_owned_by, assert_signer, create_or_allocate_account_raw,
};
use solana_program::{
    account_info::next_account_info, account_info::AccountInfo, entrypoint::ProgramResult, msg,
    program::invoke, pubkey::Pubkey, rent::Rent, system_instruction, sysvar::Sysvar,
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instruction = TrifleInstruction::try_from_slice(input)?;
    match instruction {
        TrifleInstruction::CreateEscrowConstraintModelAccount(args) => {
            msg!("Instruction: Create Escrow Constraint Model Account");
            create_escrow_contstraints_model_account(program_id, accounts, args)
        }
        TrifleInstruction::AddConstraintToEscrowConstraintModel(args) => {
            msg!("Instruction: Add Constraint To Escrow Constraint Model");
            add_constraint_to_escrow_constraint_model(program_id, accounts, args)
        }
    }
}

fn create_escrow_contstraints_model_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: CreateEscrowConstraintModelAccountArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let escrow_constraint_model_account = next_account_info(account_info_iter)?;
    let payer = next_account_info(account_info_iter)?;
    let update_authority = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent = next_account_info(account_info_iter)?;

    let escrow_constraint_model = EscrowConstraintModel {
        key: Key::EscrowConstraintModel,
        name: args.name.to_owned(),
        creator: payer.key.to_owned(),
        update_authority: update_authority.key.to_owned(),
        constraints: vec![],
        count: 0,
    };

    let bump = assert_derivation(
        program_id,
        escrow_constraint_model_account,
        &[
            ESCROW_PREFIX.as_bytes(),
            program_id.as_ref(),
            payer.key.as_ref(),
            args.name.as_bytes(),
        ],
    )?;

    let escrow_constraint_model_seeds = &[
        ESCROW_PREFIX.as_ref(),
        program_id.as_ref(),
        payer.key.as_ref(),
        args.name.as_ref(),
        &[bump],
    ];

    create_or_allocate_account_raw(
        *program_id,
        escrow_constraint_model_account,
        system_program,
        payer,
        escrow_constraint_model.try_len()?,
        escrow_constraint_model_seeds,
    )?;

    escrow_constraint_model
        .serialize(&mut *escrow_constraint_model_account.try_borrow_mut_data()?)?;

    Ok(())
}

fn add_constraint_to_escrow_constraint_model(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: AddConstraintToEscrowConstraintModelArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let escrow_constraint_model_account = next_account_info(account_info_iter)?;
    let payer = next_account_info(account_info_iter)?;
    let update_authority = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    let mut escrow_constraint_model: EscrowConstraintModel =
        EscrowConstraintModel::try_from_slice(&escrow_constraint_model_account.data.borrow())?;

    if escrow_constraint_model.update_authority != *update_authority.key {
        return Err(TrifleError::InvalidUpdateAuthority.into());
    }

    assert_derivation(
        program_id,
        escrow_constraint_model_account,
        &[
            ESCROW_PREFIX.as_bytes(),
            program_id.as_ref(),
            payer.key.as_ref(),
            escrow_constraint_model.name.as_bytes(),
        ],
    )?;

    escrow_constraint_model.constraints.push(args.constraint);

    resize_or_reallocate_account_raw(
        escrow_constraint_model_account,
        payer,
        system_program,
        escrow_constraint_model.try_len()?,
    )?;

    escrow_constraint_model
        .serialize(&mut *escrow_constraint_model_account.try_borrow_mut_data()?)?;

    Ok(())
}

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
