use crate::{
    error::MetadataError,
    instruction::MetadataInstruction,
    state::{
        EscrowConstraintModel, Metadata, TokenMetadataAccount, TokenOwnedEscrow, ESCROW_PREFIX,
        PREFIX,
    },
    utils::{assert_derivation, assert_owned_by, assert_signer, resize_or_reallocate_account_raw},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    pubkey::Pubkey,
};

#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct TransferOutOfEscrowArgs {
    pub amount: u64,
    pub index: u64,
}

pub fn transfer_out_of_escrow(
    program_id: Pubkey,
    escrow: Pubkey,
    payer: Pubkey,
    attribute_mint: Pubkey,
    attribute_src: Pubkey,
    attribute_dst: Pubkey,
    attribute_metadata: Pubkey,
    escrow_mint: Pubkey,
    escrow_account: Pubkey,
    constraint_model: Option<Pubkey>,
    amount: u64,
    index: u64,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(escrow, false),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(attribute_mint, false),
        AccountMeta::new(attribute_src, false),
        AccountMeta::new(attribute_dst, false),
        AccountMeta::new_readonly(attribute_metadata, false),
        AccountMeta::new_readonly(escrow_mint, false),
        AccountMeta::new_readonly(escrow_account, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
    ];

    if let Some(constraint_model) = constraint_model {
        accounts.push(AccountMeta::new_readonly(constraint_model, false));
    }

    let data = MetadataInstruction::TransferOutOfEscrow(TransferOutOfEscrowArgs { amount, index })
        .try_to_vec()
        .unwrap();

    Instruction {
        program_id,
        accounts,
        data,
    }
}

pub fn process_transfer_out_of_escrow(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: TransferOutOfEscrowArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let escrow_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let attribute_mint_info = next_account_info(account_info_iter)?;
    let attribute_src_info = next_account_info(account_info_iter)?;
    let attribute_dst_info = next_account_info(account_info_iter)?;
    let attribute_metadata_info = next_account_info(account_info_iter)?;
    let escrow_mint_info = next_account_info(account_info_iter)?;
    let escrow_account_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let ata_program_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let maybe_escrow_constraint_model = next_account_info(account_info_iter);

    // Owned by token-metadata program.
    assert_owned_by(attribute_metadata_info, program_id)?;
    let _attribute_metadata: Metadata = Metadata::from_account_info(attribute_metadata_info)?;

    let bump_seed = assert_derivation(
        program_id,
        escrow_info,
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            escrow_mint_info.key.as_ref(),
            ESCROW_PREFIX.as_bytes(),
        ],
    )?;

    //assert_update_authority_is_correct(&metadata, update_authority_info)?;

    // Derive the seeds for PDA signing.
    let escrow_authority_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        escrow_mint_info.key.as_ref(),
        ESCROW_PREFIX.as_bytes(),
        &[bump_seed],
    ];

    assert_signer(payer_info)?;

    // msg!("\nCreating ATA: {:#?}\n", attribute_dst_info.key);

    // Allocate the escrow accounts new ATA.
    let create_escrow_ata_ix =
        spl_associated_token_account::instruction::create_associated_token_account(
            payer_info.key,
            payer_info.key,
            attribute_mint_info.key,
        );

    // msg!("\n{:#?}\n", create_escrow_ata_ix);

    invoke(
        &create_escrow_ata_ix,
        &[
            payer_info.clone(),
            attribute_dst_info.clone(),
            attribute_mint_info.clone(),
            system_account_info.clone(),
            token_program_info.clone(),
            ata_program_info.clone(),
            rent_info.clone(),
        ],
    )?;

    // msg!("\nATA Created\n");

    // Deserialize the token accounts and perform checks.
    let attribute_src = spl_token::state::Account::unpack(&attribute_src_info.data.borrow())?;
    assert!(attribute_src.mint == *attribute_mint_info.key);
    assert!(attribute_src.delegate.is_none());
    assert!(attribute_src.amount >= args.amount);
    // msg!(
    //     "\nattribute_src_info:{:#?}\n{:#?}",
    //     attribute_src_info.key,
    //     attribute_src
    // );
    let attribute_dst = spl_token::state::Account::unpack(&attribute_dst_info.data.borrow())?;
    assert!(attribute_dst.mint == *attribute_mint_info.key);
    assert!(attribute_dst.delegate.is_none());
    // msg!(
    //     "\nattribute_dst_info:{:#?}\n{:#?}",
    //     attribute_dst_info.key,
    //     attribute_dst
    // );
    let escrow_account = spl_token::state::Account::unpack(&escrow_account_info.data.borrow())?;
    // msg!(
    //     "\nescrow_account_info:{:#?}\n{:#?}",
    //     escrow_account_info.key,
    //     escrow_account
    // );

    assert!(attribute_dst.owner == escrow_account.owner);

    // Check constraints.
    //TODO

    // Transfer the token from the current owner into the escrow.
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        attribute_src_info.key,
        attribute_dst_info.key,
        escrow_info.key,
        &[escrow_info.key],
        args.amount,
    )
    .unwrap();

    invoke_signed(
        &transfer_ix,
        &[
            attribute_src_info.clone(),
            attribute_dst_info.clone(),
            escrow_info.clone(),
            token_program_info.clone(),
        ],
        &[escrow_authority_seeds],
    )?;

    // let escrow_info_clone = escrow_info.clone();
    // let buf = &escrow_info_clone.data.borrow_mut();
    //let mut buf = escrow_info.data.borrow().as_ref();
    // let mut toe = TokenOwnedEscrow::deserialize(&mut buf.as_ref())?;

    // Update the TOE to point to the token it now owns.
    let mut toe: TokenOwnedEscrow = TokenOwnedEscrow::from_account_info(escrow_info)?;

    // if we expect a constraint model, check it
    if toe.model.is_some() {
        // check to see if a constraint model was even passed in.
        let escrow_constraint_model = maybe_escrow_constraint_model
            .map_err(|_| MetadataError::MissingEscrowConstraintModel)?;

        assert_owned_by(escrow_constraint_model, program_id)?;

        // make sure the constraint model's key matches the one set on the toe.
        if toe.model.unwrap() != *escrow_constraint_model.key {
            return Err(MetadataError::InvalidEscrowConstraintModel.into());
        }

        // deserialize the constraint model
        let model: EscrowConstraintModel =
            EscrowConstraintModel::from_account_info(escrow_constraint_model)?;

        msg!("EscrowConstraintModel: {:#?}", model);
        model.validate_at(attribute_mint_info.key, args.index as usize)?;
    }

    for token in toe.tokens.iter_mut() {
        *token = None;
    }
    resize_or_reallocate_account_raw(escrow_info, payer_info, system_account_info, toe.len())?;
    borsh::BorshSerialize::serialize(&toe, &mut *escrow_info.try_borrow_mut_data()?)?;

    Ok(())
}
