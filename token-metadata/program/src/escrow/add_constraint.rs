use crate::{
    error::MetadataError,
    instruction::MetadataInstruction,
    state::{EscrowConstraint, EscrowConstraintModel, TokenMetadataAccount, ESCROW_PREFIX, PREFIX},
    utils::{assert_derivation, resize_or_reallocate_account_raw},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct AddConstraintToEscrowConstraintModelArgs {
    pub constraint: EscrowConstraint,
}

pub fn add_constraint_to_escrow_constraint_model(
    program_id: Pubkey,
    escrow_constraint_model: Pubkey,
    payer: Pubkey,
    update_authority: Pubkey,
    constraint: EscrowConstraint,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(escrow_constraint_model, false),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(update_authority, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    let data = MetadataInstruction::AddConstraintToEscrowConstraintModel(
        AddConstraintToEscrowConstraintModelArgs { constraint },
    )
    .try_to_vec()
    .unwrap();

    Instruction {
        program_id,
        accounts,
        data,
    }
}

pub fn process_add_constraint_to_escrow_constraint_model(
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
        EscrowConstraintModel::from_account_info(escrow_constraint_model_account)?;

    if escrow_constraint_model.update_authority != *update_authority.key {
        return Err(MetadataError::UpdateAuthorityIncorrect.into());
    }

    assert_derivation(
        program_id,
        escrow_constraint_model_account,
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            ESCROW_PREFIX.as_bytes(),
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

    borsh::BorshSerialize::serialize(
        &escrow_constraint_model,
        &mut *escrow_constraint_model_account.try_borrow_mut_data()?,
    )?;

    Ok(())
}
