use crate::{
    escrow::state::{EscrowConstraintModel, ESCROW_PREFIX},
    instruction::MetadataInstruction,
    state::{Key, PREFIX},
    utils::{assert_derivation, create_or_allocate_account_raw},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    pubkey::Pubkey,
};

#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct CreateEscrowConstraintModelAccountArgs {
    pub name: String,
}

pub fn create_escrow_constraint_model(
    program_id: Pubkey,
    escrow_constraint_model: Pubkey,
    payer: Pubkey,
    update_authority: Pubkey,
    system_program: Pubkey,
    name: &str,
) -> Instruction {
    let name = name.to_owned();
    let accounts = vec![
        AccountMeta::new(escrow_constraint_model, false),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(update_authority, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let data = MetadataInstruction::CreateEscrowConstraintModelAccount(
        CreateEscrowConstraintModelAccountArgs { name },
    )
    .try_to_vec()
    .unwrap();

    Instruction {
        program_id,
        accounts,
        data,
    }
}

pub fn process_create_escrow_constraint_model_account(
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

    msg!("{:#?}", escrow_constraint_model);
    msg!("{:#?}", escrow_constraint_model.try_len()?);

    let bump = assert_derivation(
        program_id,
        escrow_constraint_model_account,
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            ESCROW_PREFIX.as_bytes(),
            payer.key.as_ref(),
            args.name.as_bytes(),
        ],
    )?;

    let escrow_constraint_model_seeds = &[
        PREFIX.as_ref(),
        program_id.as_ref(),
        ESCROW_PREFIX.as_ref(),
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

    borsh::BorshSerialize::serialize(
        &escrow_constraint_model,
        &mut *escrow_constraint_model_account.try_borrow_mut_data()?,
    )?;

    Ok(())
}
