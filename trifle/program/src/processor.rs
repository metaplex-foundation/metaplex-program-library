use std::collections::HashMap;

use crate::{
    error::TrifleError,
    instruction::{
        AddConstraintToEscrowConstraintModelArgs, CreateEscrowConstraintModelAccountArgs,
        TrifleInstruction,
    },
    state::{escrow_constraints::EscrowConstraintModel, trifle::Trifle, Key, ESCROW_PREFIX},
    util::resize_or_reallocate_account_raw,
};
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_metadata::{
    id as token_metadata_program_id,
    utils::{assert_derivation, assert_owned_by, assert_signer, create_or_allocate_account_raw},
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
        TrifleInstruction::CreateTrifleAccount => {
            msg!("Instruction: Create Trifle Account");
            create_trifle_account(program_id, accounts)
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

fn create_token_escrow() -> ProgramResult {
    Ok(())
}

fn create_trifle_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let trifle_account = next_account_info(account_info_iter)?;
    let token_escrow_account = next_account_info(account_info_iter)?;
    let token_escrow_authority = next_account_info(account_info_iter)?;
    let escrow_constraint_model_account = next_account_info(account_info_iter)?;
    let payer = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    assert_signer(payer)?;
    assert_signer(token_escrow_authority)?;
    assert_owned_by(token_escrow_account, &token_metadata_program_id())?;
    assert_owned_by(escrow_constraint_model_account, program_id)?;

    let escrow_constraint_model_key =
        Key::try_from_slice(&escrow_constraint_model_account.data.borrow()[0..1])?;

    if escrow_constraint_model_key != Key::EscrowConstraintModel {
        return Err(TrifleError::InvalidEscrowConstraintModel.into());
    }

    let trifle = Trifle {
        key: Key::Trifle,
        token_escrow: token_escrow_account.key.to_owned(),
        escrow_constraint_model: escrow_constraint_model_account.key.to_owned(),
        tokens: HashMap::new(),
    };

    let signer_seeds = &[
        ESCROW_PREFIX.as_ref(),
        program_id.as_ref(),
        payer.key.as_ref(),
        trifle.token_escrow.as_ref(),
    ];

    create_or_allocate_account_raw(
        *program_id,
        trifle_account,
        system_program,
        payer,
        trifle.try_len()?,
        signer_seeds,
    )?;

    trifle.serialize(&mut *trifle_account.try_borrow_mut_data()?)?;

    Ok(())
}

// proxy transfer_in
// proxy transfer_out
