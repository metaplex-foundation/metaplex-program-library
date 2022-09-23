use std::collections::HashMap;

use crate::{
    error::TrifleError,
    instruction::{
        AddConstraintToEscrowConstraintModelArgs, CreateEscrowConstraintModelAccountArgs,
        TrifleInstruction,
    },
    state::{
        escrow_constraints::EscrowConstraintModel, trifle::Trifle, Key, ESCROW_SEED, TRIFLE_SEED,
    },
    util::resize_or_reallocate_account_raw,
};
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_metadata::{
    id as token_metadata_program_id,
    utils::{assert_derivation, assert_owned_by, assert_signer, create_or_allocate_account_raw},
};
use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    rent::Rent,
    slot_hashes::MAX_ENTRIES,
    system_instruction,
    sysvar::Sysvar,
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

    // TODO: make account info names end in _info
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

    // TODO: seeds
    let bump = assert_derivation(
        program_id,
        escrow_constraint_model_account,
        &[
            ESCROW_SEED.as_bytes(),
            program_id.as_ref(),
            payer.key.as_ref(),
            args.name.as_bytes(),
        ],
    )?;

    // TODO: seeds
    let escrow_constraint_model_seeds = &[
        ESCROW_SEED.as_ref(),
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

    // TODO: make account info names end in _info
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
            ESCROW_SEED.as_bytes(),
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

fn create_trifle_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let escrow_info = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let edition_info = next_account_info(account_info_iter)?;
    let trifle_info = next_account_info(account_info_iter)?;
    let trifle_authority_info = next_account_info(account_info_iter)?;
    let escrow_constraint_model_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    let trifle_pda_bump = assert_derivation(
        program_id,
        trifle_info,
        &[
            TRIFLE_SEED.as_bytes(),
            mint_info.key.as_ref(),
            trifle_authority_info.key.as_ref(),
            escrow_constraint_model_info.key.as_ref(),
        ],
    )?;

    assert_signer(payer_info)?;
    assert_signer(trifle_authority_info)?;
    assert_owned_by(escrow_info, &token_metadata_program_id())?;
    assert_owned_by(escrow_constraint_model_info, program_id)?;
    assert_owned_by(metadata_info, &token_metadata_program_id())?;
    assert_owned_by(mint_info, &spl_token::id())?;
    assert_owned_by(token_account_info, &spl_token::id())?;

    let escrow_constraint_model_key =
        Key::try_from_slice(&escrow_constraint_model_info.data.borrow()[0..1])?;

    if escrow_constraint_model_key != Key::EscrowConstraintModel {
        return Err(TrifleError::InvalidEscrowConstraintModel.into());
    }

    let trifle_signer_seeds = &[
        TRIFLE_SEED.as_bytes(),
        mint_info.key.as_ref(),
        trifle_authority_info.key.as_ref(),
        escrow_constraint_model_info.key.as_ref(),
        &[trifle_pda_bump],
    ];

    let trifle = Trifle {
        key: Key::Trifle,
        token_escrow: escrow_info.key.to_owned(),
        escrow_constraint_model: escrow_constraint_model_info.key.to_owned(),
        tokens: HashMap::new(),
    };

    create_or_allocate_account_raw(
        *program_id,
        trifle_info,
        system_program_info,
        payer_info,
        trifle.try_len()?,
        trifle_signer_seeds,
    )?;

    trifle.serialize(&mut *trifle_info.try_borrow_mut_data()?)?;

    let create_escrow_account_ix = mpl_token_metadata::escrow::create_escrow_account(
        token_metadata_program_id(),
        *escrow_info.key,
        *metadata_info.key,
        *mint_info.key,
        *token_account_info.key,
        *edition_info.key,
        *payer_info.key,
        Some(*trifle_info.key),
    );

    let account_infos = vec![
        escrow_info.clone(),
        metadata_info.clone(),
        mint_info.clone(),
        token_account_info.clone(),
        edition_info.clone(),
        payer_info.clone(),
        system_program_info.clone(),
        trifle_info.clone(),
    ];

    invoke_signed(
        &create_escrow_account_ix,
        &account_infos,
        &[trifle_signer_seeds],
    )?;

    Ok(())
}

// proxy transfer_in
// proxy transfer_out
