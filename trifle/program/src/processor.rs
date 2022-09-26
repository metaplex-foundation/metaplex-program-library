use std::collections::HashMap;

use crate::{
    error::TrifleError,
    instruction::{
        AddConstraintToEscrowConstraintModelArgs, CreateEscrowConstraintModelAccountArgs,
        TransferInArgs, TransferOutArgs, TrifleInstruction,
    },
    state::{
        escrow_constraints::EscrowConstraintModel, trifle::Trifle, Key, SolanaAccount, ESCROW_SEED,
        TRIFLE_SEED,
    },
    util::resize_or_reallocate_account_raw,
};
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_metadata::{
    error::MetadataError,
    id as token_metadata_program_id,
    state::{EscrowAuthority, Metadata, TokenMetadataAccount, ESCROW_PREFIX, PREFIX},
    utils::{assert_derivation, assert_owned_by, assert_signer, create_or_allocate_account_raw},
};
use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_memory::sol_memcpy,
    program_pack::Pack,
    pubkey::Pubkey,
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
            create_escrow_constraints_model_account(program_id, accounts, args)
        }
        TrifleInstruction::AddConstraintToEscrowConstraintModel(args) => {
            msg!("Instruction: Add Constraint To Escrow Constraint Model");
            add_constraint_to_escrow_constraint_model(program_id, accounts, args)
        }
        TrifleInstruction::CreateTrifleAccount => {
            msg!("Instruction: Create Trifle Account");
            create_trifle_account(program_id, accounts)
        }
        TrifleInstruction::TransferIn(args) => {
            msg!("Instruction: Transfer In");
            transfer_in(program_id, accounts, args)
        }
        TrifleInstruction::TransferOut(args) => {
            msg!("Instruction: Transfer Out");
            transfer_out(program_id, accounts, args)
        }
    }
}

fn create_escrow_constraints_model_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: CreateEscrowConstraintModelAccountArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let escrow_constraint_model_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    let escrow_constraint_model = EscrowConstraintModel {
        key: Key::EscrowConstraintModel,
        name: args.name.to_owned(),
        creator: payer_info.key.to_owned(),
        update_authority: update_authority_info.key.to_owned(),
        ..Default::default()
    };

    let bump = assert_derivation(
        program_id,
        escrow_constraint_model_info,
        &[
            ESCROW_SEED.as_bytes(),
            payer_info.key.as_ref(),
            args.name.as_bytes(),
        ],
    )?;

    let escrow_constraint_model_seeds = &[
        ESCROW_SEED.as_ref(),
        payer_info.key.as_ref(),
        args.name.as_ref(),
        &[bump],
    ];

    create_or_allocate_account_raw(
        *program_id,
        escrow_constraint_model_info,
        system_program_info,
        payer_info,
        escrow_constraint_model.try_len()?,
        escrow_constraint_model_seeds,
    )?;

    escrow_constraint_model.serialize(&mut *escrow_constraint_model_info.try_borrow_mut_data()?)?;

    Ok(())
}

fn add_constraint_to_escrow_constraint_model(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: AddConstraintToEscrowConstraintModelArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let escrow_constraint_model_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    assert_owned_by(escrow_constraint_model_info, program_id)?;
    assert_signer(payer_info)?;
    assert_signer(update_authority_info)?;

    let mut escrow_constraint_model: EscrowConstraintModel =
        EscrowConstraintModel::try_from_slice(&escrow_constraint_model_info.data.borrow())?;

    if escrow_constraint_model.update_authority != *update_authority_info.key {
        return Err(TrifleError::InvalidUpdateAuthority.into());
    }

    assert_derivation(
        program_id,
        escrow_constraint_model_info,
        &[
            ESCROW_SEED.as_bytes(),
            payer_info.key.as_ref(),
            escrow_constraint_model.name.as_bytes(),
        ],
    )?;

    if escrow_constraint_model
        .constraints
        .contains_key(&args.constraint_name)
    {
        return Err(TrifleError::ConstraintAlreadyExists.into());
    }

    escrow_constraint_model
        .constraints
        .insert(args.constraint_name, args.constraint);

    resize_or_reallocate_account_raw(
        escrow_constraint_model_info,
        payer_info,
        system_program_info,
        escrow_constraint_model.try_len()?,
    )?;

    escrow_constraint_model.serialize(&mut *escrow_constraint_model_info.try_borrow_mut_data()?)?;

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
    let _tm_program_info = next_account_info(account_info_iter)?;
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

    msg!("Checking singers.");
    assert_signer(payer_info)?;
    assert_signer(trifle_authority_info)?;

    msg!("Checking escrow account.");
    assert_owned_by(escrow_info, system_program_info.key)?;
    if !escrow_info.data_is_empty() {
        return Err(MetadataError::AlreadyInitialized.into());
    }

    msg!("Checking escrow_constraint_model_info.");
    assert_owned_by(escrow_constraint_model_info, program_id)?;

    msg!("Checking metadata_info.");
    assert_owned_by(metadata_info, &token_metadata_program_id())?;

    msg!("Checking mint_info.");
    assert_owned_by(mint_info, &spl_token::id())?;

    msg!("Checking token_account_info.");
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

    let serialized_data = trifle.try_to_vec().unwrap();
    create_or_allocate_account_raw(
        *program_id,
        trifle_info,
        system_program_info,
        payer_info,
        serialized_data.len(),
        trifle_signer_seeds,
    )?;

    //trifle.serialize(&mut *trifle_info.try_borrow_mut_data()?)?;
    sol_memcpy(
        &mut **trifle_info.try_borrow_mut_data().unwrap(),
        &serialized_data,
        serialized_data.len(),
    );

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

    msg!("Creating token escrow.");
    invoke_signed(
        &create_escrow_account_ix,
        &account_infos,
        &[trifle_signer_seeds],
    )?;

    Ok(())
}

fn transfer_in(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: TransferInArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let trifle_account = next_account_info(account_info_iter)?;
    let constraint_model_info = next_account_info(account_info_iter)?;
    let escrow_account = next_account_info(account_info_iter)?;
    let payer = next_account_info(account_info_iter)?;
    let _trifle_authority = next_account_info(account_info_iter)?;
    let attribute_mint = next_account_info(account_info_iter)?;
    let attribute_src_token_account = next_account_info(account_info_iter)?;
    let attribute_dst_token_account = next_account_info(account_info_iter)?;
    let attribute_metadata = next_account_info(account_info_iter)?;
    let escrow_mint = next_account_info(account_info_iter)?;
    let _escrow_token_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let _ata_program = next_account_info(account_info_iter)?;
    let spl_token_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    assert_owned_by(attribute_metadata, &mpl_token_metadata::id())?;
    let _attribute_metadata: Metadata = Metadata::from_account_info(attribute_metadata)?;

    let tm_pid = mpl_token_metadata::id();
    let mut escrow_seeds = vec![PREFIX.as_bytes(), tm_pid.as_ref(), escrow_mint.key.as_ref()];

    let escrow_auth = EscrowAuthority::Creator(*trifle_account.key);
    for seed in escrow_auth.to_seeds() {
        escrow_seeds.push(seed);
    }

    escrow_seeds.push(ESCROW_PREFIX.as_bytes());

    let bump_seed = assert_derivation(&tm_pid, escrow_account, &escrow_seeds)?;

    assert_signer(payer)?;
    //assert_signer(trifle_authority)?;

    let constraint_model =
        EscrowConstraintModel::try_from_slice(&constraint_model_info.data.borrow())
            .map_err(|_| TrifleError::InvalidEscrowConstraintModel)?;

    constraint_model.validate(attribute_mint.key, args.slot.to_owned())?;

    // Allocate the escrow accounts new ATA.
    let create_escrow_ata_ix =
        spl_associated_token_account::instruction::create_associated_token_account(
            payer.key,
            escrow_account.key,
            attribute_mint.key,
            spl_token_program.key,
        );

    invoke(
        &create_escrow_ata_ix,
        &[
            payer.clone(),
            attribute_dst_token_account.clone(),
            escrow_account.clone(),
            attribute_mint.clone(),
            system_program.clone(),
            spl_token_program.clone(),
            rent_info.clone(),
        ],
    )?;

    // Deserialize the token accounts and perform checks.
    let attribute_src =
        spl_token::state::Account::unpack(&attribute_src_token_account.data.borrow())?;
    assert!(attribute_src.mint == *attribute_mint.key);
    assert!(attribute_src.delegate.is_none());
    assert!(attribute_src.amount >= args.amount);
    let attribute_dst =
        spl_token::state::Account::unpack(&attribute_dst_token_account.data.borrow())?;
    assert!(attribute_dst.mint == *attribute_mint.key);
    assert!(attribute_dst.delegate.is_none());
    //let escrow_account = spl_token::state::Account::unpack(&escrow_token_account.data.borrow())?;

    // TODO: Check the constraints

    // Transfer the token from the current owner into the escrow.
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        attribute_src_token_account.key,
        attribute_dst_token_account.key,
        payer.key,
        &[payer.key],
        args.amount,
    )
    .unwrap();

    invoke(
        &transfer_ix,
        &[
            attribute_src_token_account.clone(),
            attribute_dst_token_account.clone(),
            payer.clone(),
            spl_token_program.clone(),
        ],
    )?;

    // Update the Trifle account
    let mut trifle = Trifle::from_account_info(trifle_account)?;
    let token_list = trifle.tokens.get_mut(&args.slot);
    match token_list {
        Some(tokens) => {
            tokens.push(*attribute_mint.key);
        }
        None => {
            trifle.tokens.insert(args.slot, vec![*attribute_mint.key]);
        }
    }

    let serialized_data = trifle.try_to_vec().unwrap();

    resize_or_reallocate_account_raw(trifle_account, payer, system_program, serialized_data.len())?;

    sol_memcpy(
        &mut **trifle_account.try_borrow_mut_data().unwrap(),
        &serialized_data,
        serialized_data.len(),
    );

    Ok(())
}

fn transfer_out(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: TransferOutArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let trifle_account = next_account_info(account_info_iter)?;
    let constraint_model_info = next_account_info(account_info_iter)?;
    let escrow_account = next_account_info(account_info_iter)?;
    let payer = next_account_info(account_info_iter)?;
    let trifle_authority = next_account_info(account_info_iter)?;
    let attribute_mint = next_account_info(account_info_iter)?;
    let attribute_src_token_account = next_account_info(account_info_iter)?;
    let attribute_dst_token_account = next_account_info(account_info_iter)?;
    let attribute_metadata = next_account_info(account_info_iter)?;
    let escrow_mint = next_account_info(account_info_iter)?;
    let escrow_token_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let _ata_program = next_account_info(account_info_iter)?;
    let spl_token_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    assert_owned_by(attribute_metadata, &mpl_token_metadata::id())?;
    let _attribute_metadata: Metadata = Metadata::from_account_info(attribute_metadata)?;

    let tm_pid = mpl_token_metadata::id();
    let mut escrow_seeds = vec![PREFIX.as_bytes(), tm_pid.as_ref(), escrow_mint.key.as_ref()];

    let escrow_auth = EscrowAuthority::Creator(*trifle_account.key);
    for seed in escrow_auth.to_seeds() {
        escrow_seeds.push(seed);
    }

    escrow_seeds.push(ESCROW_PREFIX.as_bytes());

    let bump_seed = assert_derivation(program_id, escrow_account, &escrow_seeds)?;

    // Derive the seeds for PDA signing.
    let trifle_authority_seeds = &[
        TRIFLE_SEED.as_bytes(),
        escrow_mint.key.as_ref(),
        trifle_authority.key.as_ref(),
        constraint_model_info.key.as_ref(),
        &[bump_seed],
    ];

    assert_signer(payer)?;
    //assert_signer(trifle_authority)?;

    // Allocate the escrow accounts new ATA.
    let create_escrow_ata_ix =
        spl_associated_token_account::instruction::create_associated_token_account(
            payer.key,
            escrow_account.key,
            attribute_mint.key,
            spl_token_program.key,
        );

    invoke(
        &create_escrow_ata_ix,
        &[
            payer.clone(),
            attribute_dst_token_account.clone(),
            escrow_account.clone(),
            attribute_mint.clone(),
            system_program.clone(),
            spl_token_program.clone(),
            rent_info.clone(),
        ],
    )?;

    // Deserialize the token accounts and perform checks.

    let escrow_token_account_data =
        spl_token::state::Account::unpack(&escrow_token_account.data.borrow())?;
    // Only the parent NFT holder can transfer out
    assert!(escrow_token_account_data.owner == *payer.key);

    // TODO: Check the constraints

    // Transfer the token out of the escrow
    let transfer_ix = mpl_token_metadata::escrow::transfer_out_of_escrow(
        mpl_token_metadata::id(),
        *escrow_account.key,
        *payer.key,
        *attribute_mint.key,
        *attribute_src_token_account.key,
        *attribute_dst_token_account.key,
        *attribute_metadata.key,
        *escrow_mint.key,
        *escrow_token_account.key,
        Some(*trifle_authority.key),
        args.amount,
    );

    invoke_signed(
        &transfer_ix,
        &[
            escrow_account.clone(),
            payer.clone(),
            attribute_mint.clone(),
            attribute_src_token_account.clone(),
            attribute_dst_token_account.clone(),
            attribute_metadata.clone(),
            escrow_mint.clone(),
            escrow_token_account.clone(),
            trifle_authority.clone(),
        ],
        &[trifle_authority_seeds],
    )?;

    // Update the Trifle account
    let mut trifle = Trifle::from_account_info(trifle_account)?;
    let index = trifle
        .tokens
        .get(&args.slot)
        .unwrap()
        .iter()
        .position(|&x| x == *attribute_mint.key)
        .unwrap();
    trifle
        .tokens
        .get_mut(&args.slot)
        .unwrap()
        .swap_remove(index);

    let serialized_data = trifle.try_to_vec().unwrap();

    resize_or_reallocate_account_raw(trifle_account, payer, system_program, serialized_data.len())?;

    sol_memcpy(
        &mut **trifle_account.try_borrow_mut_data().unwrap(),
        &serialized_data,
        serialized_data.len(),
    );

    Ok(())
}
