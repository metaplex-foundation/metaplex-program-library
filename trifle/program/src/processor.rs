use crate::{
    error::TrifleError,
    instruction::{
        AddCollectionConstraintToEscrowConstraintModelArgs,
        AddNoneConstraintToEscrowConstraintModelArgs,
        AddTokensConstraintToEscrowConstraintModelArgs, CreateEscrowConstraintModelAccountArgs,
        RemoveConstraintFromEscrowConstraintModelArgs, SetRoyaltiesArgs, TransferInArgs,
        TransferOutArgs, TrifleInstruction, WithdrawRoyaltiesArgs,
    },
    state::{
        escrow_constraints::{
            EscrowConstraint, EscrowConstraintModel, EscrowConstraintType, RoyaltyInstruction,
        },
        transfer_effects::TransferEffects,
        trifle::Trifle,
        Key, SolanaAccount, ESCROW_SEED, TRIFLE_SEED,
    },
    util::{
        assert_holder, is_creation_instruction, pay_royalties, resize_or_reallocate_account_raw,
    },
};
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_metadata::{
    assertions::{assert_derivation, assert_owned_by},
    error::MetadataError,
    id as token_metadata_program_id,
    state::{EscrowAuthority, Metadata, TokenMetadataAccount, ESCROW_POSTFIX, PREFIX},
    utils::is_print_edition,
};
use mpl_utils::{assert_signer, create_or_allocate_account_raw};
use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_memory::sol_memcpy,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::{
        instructions::{get_instruction_relative, load_current_index_checked},
        Sysvar,
    },
};
use spl_token::state::{Account, Mint};

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
        TrifleInstruction::AddNoneConstraintToEscrowConstraintModel(args) => {
            msg!("Instruction: Add None Constraint To Escrow Constraint Model");
            add_none_constraint_to_escrow_constraint_model(program_id, accounts, args)
        }
        TrifleInstruction::AddCollectionConstraintToEscrowConstraintModel(args) => {
            msg!("Instruction: Add Collection Constraint To Escrow Constraint Model");
            add_collection_constraint_to_escrow_constraint_model(program_id, accounts, args)
        }
        TrifleInstruction::AddTokensConstraintToEscrowConstraintModel(args) => {
            msg!("Instruction: Add Tokens Constraint To Escrow Constraint Model");
            add_tokens_constraint_to_escrow_constraint_model(program_id, accounts, args)
        }
        TrifleInstruction::RemoveConstraintFromEscrowConstraintModel(args) => {
            msg!("Instruction: Remove Constraint From Escrow Constraint Model");
            remove_constraint_from_escrow_constraint_model(program_id, accounts, args)
        }
        TrifleInstruction::SetRoyalties(args) => {
            msg!("Instruction: Set Royalties");
            set_royalties(program_id, accounts, args)
        }
        TrifleInstruction::WithdrawRoyalties(args) => {
            msg!("Instruction: Withdraw Royalties");
            withdraw_royalties(program_id, accounts, args)
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

    let mut escrow_constraint_model = EscrowConstraintModel {
        key: Key::EscrowConstraintModel,
        name: args.name.to_owned(),
        creator: payer_info.key.to_owned(),
        update_authority: update_authority_info.key.to_owned(),
        schema_uri: args.schema_uri.to_owned(),
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

    pay_royalties(
        RoyaltyInstruction::CreateModel,
        &mut escrow_constraint_model,
        payer_info,
        escrow_constraint_model_info,
        system_program_info,
    )?;

    let serialized_data = escrow_constraint_model
        .try_to_vec()
        .map_err(|_| TrifleError::FailedToSerialize)?;

    create_or_allocate_account_raw(
        *program_id,
        escrow_constraint_model_info,
        system_program_info,
        payer_info,
        serialized_data.len(),
        escrow_constraint_model_seeds,
    )?;

    sol_memcpy(
        &mut **escrow_constraint_model_info
            .try_borrow_mut_data()
            .map_err(|_| TrifleError::FailedToBorrowAccountData)?,
        &serialized_data,
        serialized_data.len(),
    );

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
    let sysvar_ix_account_info = next_account_info(account_info_iter)?;

    let trifle_pda_bump = assert_derivation(
        program_id,
        trifle_info,
        &[
            TRIFLE_SEED.as_bytes(),
            mint_info.key.as_ref(),
            trifle_authority_info.key.as_ref(),
        ],
    )?;

    assert_signer(payer_info)?;
    assert_signer(trifle_authority_info)?;
    assert_owned_by(escrow_info, system_program_info.key)?;
    if !escrow_info.data_is_empty() {
        return Err(MetadataError::AlreadyInitialized.into());
    }
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
        &[trifle_pda_bump],
    ];

    let trifle = Trifle {
        token_escrow: escrow_info.key.to_owned(),
        escrow_constraint_model: escrow_constraint_model_info.key.to_owned(),
        ..Default::default()
    };

    let mut constraint_model =
        EscrowConstraintModel::try_from_slice(&escrow_constraint_model_info.data.borrow())
            .map_err(|_| TrifleError::InvalidEscrowConstraintModel)?;
    pay_royalties(
        RoyaltyInstruction::CreateTrifle,
        &mut constraint_model,
        payer_info,
        escrow_constraint_model_info,
        system_program_info,
    )?;

    let serialized_data = constraint_model
        .try_to_vec()
        .map_err(|_| TrifleError::FailedToSerialize)?;

    sol_memcpy(
        &mut **escrow_constraint_model_info
            .try_borrow_mut_data()
            .map_err(|_| TrifleError::FailedToBorrowAccountData)?,
        &serialized_data,
        serialized_data.len(),
    );

    let serialized_data = trifle
        .try_to_vec()
        .map_err(|_| TrifleError::FailedToSerialize)?;

    create_or_allocate_account_raw(
        *program_id,
        trifle_info,
        system_program_info,
        payer_info,
        serialized_data.len(),
        trifle_signer_seeds,
    )?;

    sol_memcpy(
        &mut **trifle_info
            .try_borrow_mut_data()
            .map_err(|_| TrifleError::FailedToBorrowAccountData)?,
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
        sysvar_ix_account_info.clone(),
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

    let trifle_info = next_account_info(account_info_iter)?;
    let trifle_authority_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let constraint_model_info = next_account_info(account_info_iter)?;
    let escrow_info = next_account_info(account_info_iter)?;
    let escrow_mint_info = next_account_info(account_info_iter)?;
    let escrow_token_info = next_account_info(account_info_iter)?;
    let escrow_edition_info = next_account_info(account_info_iter)?;
    let attribute_mint_info = next_account_info(account_info_iter)?;
    let attribute_src_token_info = next_account_info(account_info_iter)?;
    let attribute_dst_token_info = next_account_info(account_info_iter)?;
    let attribute_metadata_info = next_account_info(account_info_iter)?;
    let attribute_edition_info = next_account_info(account_info_iter)?;
    let attribute_collection_metadata_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let _associated_token_account_program_info = next_account_info(account_info_iter)?;
    let token_metadata_program_info = next_account_info(account_info_iter)?;

    assert_signer(payer_info)?;
    assert_owned_by(attribute_metadata_info, token_metadata_program_info.key)?;

    let escrow_token_account_data = Account::unpack(&escrow_token_info.data.borrow())?;
    // Only the parent NFT holder can transfer in
    assert_holder(&escrow_token_account_data, payer_info)?;

    let attribute_metadata: Metadata = Metadata::from_account_info(attribute_metadata_info)?;
    let mut escrow_seeds = vec![
        PREFIX.as_bytes(),
        token_metadata_program_info.key.as_ref(),
        escrow_mint_info.key.as_ref(),
    ];

    let escrow_auth = EscrowAuthority::Creator(*trifle_info.key);
    for seed in escrow_auth.to_seeds() {
        escrow_seeds.push(seed);
    }

    escrow_seeds.push(ESCROW_POSTFIX.as_bytes());

    assert_derivation(token_metadata_program_info.key, escrow_info, &escrow_seeds)?;

    // Deserialize the token accounts and perform checks.
    let attribute_src = Account::unpack(&attribute_src_token_info.data.borrow())?;
    assert!(attribute_src.mint == *attribute_mint_info.key);
    assert!(attribute_src.delegate.is_none());
    assert!(attribute_src.amount >= args.amount);

    // TODO: perform assertions on attribute_dst if it exists.
    // let attribute_dst =
    //     spl_token::state::Account::unpack(&attribute_dst_token_account.data.borrow())?;
    // msg!("past second unpack");
    // assert!(attribute_dst.mint == *attribute_mint.key);
    // assert!(attribute_dst.delegate.is_none());
    // msg!("past explicit assertions.");

    let trifle_seeds = &[
        TRIFLE_SEED.as_bytes(),
        escrow_mint_info.key.as_ref(),
        trifle_authority_info.key.as_ref(),
    ];

    let trifle_bump_seed = assert_derivation(program_id, trifle_info, trifle_seeds)?;
    let trifle_signer_seeds = &[
        TRIFLE_SEED.as_bytes(),
        escrow_mint_info.key.as_ref(),
        trifle_authority_info.key.as_ref(),
        &[trifle_bump_seed],
    ];

    let mut constraint_model =
        EscrowConstraintModel::try_from_slice(&constraint_model_info.data.borrow())
            .map_err(|_| TrifleError::InvalidEscrowConstraintModel)?;

    let constraint = constraint_model
        .constraints
        .get(&args.slot)
        .ok_or(TrifleError::InvalidEscrowConstraint)?;

    if let EscrowConstraintType::Collection(_) = constraint.constraint_type {
        let collection_key = attribute_metadata
            .collection
            .clone()
            .ok_or(TrifleError::InvalidCollection)?
            .key;

        constraint_model.validate(&collection_key, &args.slot)?;
    } else {
        constraint_model.validate(attribute_mint_info.key, &args.slot)?;
    }

    let transfer_effects = TransferEffects::from(constraint.transfer_effects);

    // check fuse options
    if transfer_effects.burn() && transfer_effects.freeze() {
        msg!("Transfer effects cannot be both burn and freeze");
        return Err(TrifleError::TransferEffectConflict.into());
    }

    // If burn is not set, create an ATA for the incoming token and perform the transfer.
    if !transfer_effects.burn() {
        // Allocate the escrow accounts new ATA.
        let create_escrow_ata_ix =
            spl_associated_token_account::instruction::create_associated_token_account(
                payer_info.key,
                escrow_info.key,
                attribute_mint_info.key,
            );

        invoke(
            &create_escrow_ata_ix,
            &[
                payer_info.clone(),
                attribute_dst_token_info.clone(),
                escrow_info.clone(),
                attribute_mint_info.clone(),
                system_program_info.clone(),
                token_program_info.clone(),
            ],
        )?;

        // Transfer the token from the current owner into the escrow.
        let transfer_ix = spl_token::instruction::transfer(
            &spl_token::id(),
            attribute_src_token_info.key,
            attribute_dst_token_info.key,
            payer_info.key,
            &[payer_info.key],
            args.amount,
        )?;

        invoke(
            &transfer_ix,
            &[
                attribute_src_token_info.clone(),
                attribute_dst_token_info.clone(),
                payer_info.clone(),
                token_program_info.clone(),
            ],
        )?;
    } else {
        let attribute_mint = Mint::unpack(&attribute_mint_info.data.borrow())?;
        if is_print_edition(
            attribute_edition_info,
            attribute_mint.decimals,
            attribute_mint.supply,
        ) {
            return Err(TrifleError::CannotBurnPrintEdition.into());
        }

        let maybe_collection_metadata_pubkey = if attribute_metadata.collection.is_some() {
            Metadata::from_account_info(attribute_collection_metadata_info)
                .map_err(|_| TrifleError::InvalidCollectionMetadata)?;

            Some(*attribute_collection_metadata_info.key)
        } else {
            None
        };

        // Burn the token from the current owner.
        let burn_ix = mpl_token_metadata::instruction::burn_nft(
            mpl_token_metadata::id(),
            *attribute_metadata_info.key,
            *payer_info.key,
            *attribute_mint_info.key,
            *attribute_src_token_info.key,
            *attribute_edition_info.key,
            *token_program_info.key,
            maybe_collection_metadata_pubkey,
        );

        let mut accounts = vec![
            attribute_metadata_info.clone(),
            payer_info.clone(),
            attribute_mint_info.clone(),
            attribute_src_token_info.clone(),
            attribute_edition_info.clone(),
            token_program_info.clone(),
        ];

        if maybe_collection_metadata_pubkey.is_some() {
            accounts.push(attribute_collection_metadata_info.clone());
        }

        // invoke_signed(&burn_ix, &accounts, &[trifle_signer_seeds])?;
        invoke(&burn_ix, &accounts)?;
    }

    if transfer_effects.freeze_parent() {
        // make sure the freeze authority is set
        let escrow_mint = Mint::unpack(&escrow_mint_info.data.borrow())?;

        if escrow_mint.freeze_authority.is_none() {
            msg!("Freeze authority is not set");
            return Err(TrifleError::FreezeAuthorityNotSet.into());
        }

        let freeze_ix = mpl_token_metadata::instruction::freeze_delegated_account(
            mpl_token_metadata::id(),
            *trifle_info.key,
            *escrow_token_info.key,
            *escrow_edition_info.key,
            *escrow_mint_info.key,
        );

        let accounts = &[
            trifle_info.clone(),
            escrow_token_info.clone(),
            escrow_edition_info.clone(),
            escrow_mint_info.clone(),
            token_program_info.clone(),
        ];

        invoke_signed(&freeze_ix, accounts, &[trifle_signer_seeds])?;
    }

    if transfer_effects.track() {
        let mut trifle = Trifle::from_account_info(trifle_info)?;

        trifle.try_add(constraint, args.slot, *attribute_mint_info.key, args.amount)?;

        let serialized_data = trifle
            .try_to_vec()
            .map_err(|_| TrifleError::FailedToSerialize)?;

        resize_or_reallocate_account_raw(
            trifle_info,
            payer_info,
            system_program_info,
            serialized_data.len(),
        )?;

        sol_memcpy(
            &mut **trifle_info
                .try_borrow_mut_data()
                .map_err(|_| TrifleError::FailedToBorrowAccountData)?,
            &serialized_data,
            serialized_data.len(),
        );
    }

    // collect and track royalties
    pay_royalties(
        RoyaltyInstruction::TransferIn,
        &mut constraint_model,
        payer_info,
        constraint_model_info,
        system_program_info,
    )?;

    // save constraint model
    let serialized_data = constraint_model
        .try_to_vec()
        .map_err(|_| TrifleError::FailedToSerialize)?;

    sol_memcpy(
        &mut **constraint_model_info
            .try_borrow_mut_data()
            .map_err(|_| TrifleError::FailedToBorrowAccountData)?,
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

    let trifle_info = next_account_info(account_info_iter)?;
    let constraint_model_info = next_account_info(account_info_iter)?;
    let escrow_info = next_account_info(account_info_iter)?;
    let escrow_token_info = next_account_info(account_info_iter)?;
    let escrow_mint_info = next_account_info(account_info_iter)?;
    let escrow_metadata_info = next_account_info(account_info_iter)?;
    let escrow_edition_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let trifle_authority_info = next_account_info(account_info_iter)?;
    let attribute_mint_info = next_account_info(account_info_iter)?;
    let attribute_src_token_info = next_account_info(account_info_iter)?;
    let attribute_dst_token_info = next_account_info(account_info_iter)?;
    let attribute_metadata_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let _ata_program_info = next_account_info(account_info_iter)?;
    let _spl_token_program_info = next_account_info(account_info_iter)?;
    let token_metadata_program_info = next_account_info(account_info_iter)?;
    let sysvar_ix_account_info = next_account_info(account_info_iter)?;

    assert_owned_by(attribute_metadata_info, &mpl_token_metadata::id())?;
    let _attribute_metadata: Metadata = Metadata::from_account_info(attribute_metadata_info)?;

    let mut escrow_seeds = vec![
        PREFIX.as_bytes(),
        token_metadata_program_info.key.as_ref(),
        escrow_mint_info.key.as_ref(),
    ];

    let escrow_auth = EscrowAuthority::Creator(*trifle_info.key);
    for seed in escrow_auth.to_seeds() {
        escrow_seeds.push(seed);
    }

    escrow_seeds.push(ESCROW_POSTFIX.as_bytes());
    assert_derivation(token_metadata_program_info.key, escrow_info, &escrow_seeds)?;

    let trifle_seeds = &[
        TRIFLE_SEED.as_bytes(),
        escrow_mint_info.key.as_ref(),
        trifle_authority_info.key.as_ref(),
    ];

    let trifle_bump_seed = assert_derivation(program_id, trifle_info, trifle_seeds)?;

    // Derive the seeds for PDA signing.
    let trifle_signer_seeds = &[
        TRIFLE_SEED.as_bytes(),
        escrow_mint_info.key.as_ref(),
        trifle_authority_info.key.as_ref(),
        &[trifle_bump_seed],
    ];

    assert_signer(payer_info)?;
    // assert_signer(trifle_authority_info)?;

    let escrow_token_account_data = Account::unpack(&escrow_token_info.data.borrow())?;
    // Only the parent NFT holder can transfer out
    assert_holder(&escrow_token_account_data, payer_info)?;

    // Transfer the token out of the escrow
    let transfer_ix = mpl_token_metadata::escrow::transfer_out_of_escrow(
        *token_metadata_program_info.key,
        *escrow_info.key,
        *escrow_metadata_info.key,
        *payer_info.key,
        *attribute_mint_info.key,
        *attribute_src_token_info.key,
        *attribute_dst_token_info.key,
        *escrow_mint_info.key,
        *escrow_token_info.key,
        Some(*trifle_info.key),
        args.amount,
    );

    invoke_signed(
        &transfer_ix,
        &[
            escrow_info.clone(),
            payer_info.clone(),
            attribute_mint_info.clone(),
            attribute_src_token_info.clone(),
            attribute_dst_token_info.clone(),
            attribute_metadata_info.clone(),
            escrow_mint_info.clone(),
            escrow_token_info.clone(),
            trifle_info.clone(),
            escrow_metadata_info.clone(),
            sysvar_ix_account_info.clone(),
        ],
        &[trifle_signer_seeds],
    )?;

    // Update the Trifle account
    let mut trifle = Trifle::from_account_info(trifle_info)?;
    trifle.try_remove(args.slot.clone(), *attribute_mint_info.key, args.amount)?;

    let mut constraint_model =
        EscrowConstraintModel::try_from_slice(&constraint_model_info.data.borrow())
            .map_err(|_| TrifleError::InvalidEscrowConstraintModel)?;

    // collect fees and save the model.
    pay_royalties(
        RoyaltyInstruction::TransferOut,
        &mut constraint_model,
        payer_info,
        constraint_model_info,
        system_program_info,
    )?;

    let serialized_data = constraint_model
        .try_to_vec()
        .map_err(|_| TrifleError::FailedToSerialize)?;

    sol_memcpy(
        &mut **constraint_model_info
            .try_borrow_mut_data()
            .map_err(|_| TrifleError::FailedToBorrowAccountData)?,
        &serialized_data,
        serialized_data.len(),
    );

    let serialized_data = trifle
        .try_to_vec()
        .map_err(|_| TrifleError::FailedToSerialize)?;

    resize_or_reallocate_account_raw(
        trifle_info,
        payer_info,
        system_program_info,
        serialized_data.len(),
    )?;

    sol_memcpy(
        &mut trifle_info
            .try_borrow_mut_data()
            .map_err(|_| TrifleError::FailedToBorrowAccountData)?,
        &serialized_data,
        serialized_data.len(),
    );

    if trifle.is_empty() {
        let constraint_model =
            EscrowConstraintModel::try_from_slice(&constraint_model_info.data.borrow())
                .map_err(|_| TrifleError::InvalidEscrowConstraintModel)?;

        let constraint = constraint_model
            .constraints
            .get(&args.slot)
            .ok_or(TrifleError::InvalidEscrowConstraint)?;

        let transfer_effects = TransferEffects::from(constraint.transfer_effects);

        if transfer_effects.freeze_parent() {
            let escrow_token = Account::unpack(&escrow_token_info.data.borrow())?;
            if escrow_token.is_frozen() {
                msg!("Last token transferred out of escrow. Unfreezing the escrow token account.");

                let thaw_ix = mpl_token_metadata::instruction::thaw_delegated_account(
                    mpl_token_metadata::id(),
                    *trifle_info.key,
                    *escrow_token_info.key,
                    *escrow_edition_info.key,
                    *escrow_mint_info.key,
                );

                invoke_signed(
                    &thaw_ix,
                    &[
                        trifle_info.to_owned(),
                        escrow_token_info.to_owned(),
                        escrow_edition_info.to_owned(),
                        escrow_mint_info.to_owned(),
                        _spl_token_program_info.to_owned(),
                    ],
                    &[trifle_signer_seeds],
                )?;
            }
        }
    }

    Ok(())
}

fn add_constraint_to_escrow_constraint_model(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    take_fees: bool,
    constraint_name: String,
    escrow_constraint: EscrowConstraint,
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
        .contains_key(&constraint_name)
    {
        return Err(TrifleError::ConstraintAlreadyExists.into());
    }

    escrow_constraint_model
        .constraints
        .insert(constraint_name, escrow_constraint);

    // Pay royalties and protocol fees if we haven't already.
    if take_fees {
        pay_royalties(
            RoyaltyInstruction::AddConstraint,
            &mut escrow_constraint_model,
            payer_info,
            escrow_constraint_model_info,
            system_program_info,
        )?;
    }

    let serialized_data = escrow_constraint_model
        .try_to_vec()
        .map_err(|_| TrifleError::FailedToSerialize)?;

    resize_or_reallocate_account_raw(
        escrow_constraint_model_info,
        payer_info,
        system_program_info,
        serialized_data.len(),
    )?;

    sol_memcpy(
        &mut **escrow_constraint_model_info
            .try_borrow_mut_data()
            .map_err(|_| TrifleError::FailedToBorrowAccountData)?,
        &serialized_data,
        serialized_data.len(),
    );

    Ok(())
}

fn add_none_constraint_to_escrow_constraint_model(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: AddNoneConstraintToEscrowConstraintModelArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    accounts_iter.next(); // skip the escrow constraint model
    accounts_iter.next(); // skip the payer
    accounts_iter.next(); // skip the update authority
    accounts_iter.next(); // skip the system program
    let sysvar_instruction_info = next_account_info(accounts_iter)?;

    let constraint = EscrowConstraint {
        constraint_type: EscrowConstraintType::None,
        token_limit: args.token_limit,
        transfer_effects: args.transfer_effects,
    };

    // Check if the previous instruction was a creation instruction, so we don't double-charge protocol fees.
    let mut creation_ix = false;
    if load_current_index_checked(sysvar_instruction_info)? > 0 {
        let prev_ix = get_instruction_relative(-1, sysvar_instruction_info)?;
        creation_ix = is_creation_instruction(*prev_ix.data.first().unwrap_or(&255));
    }

    add_constraint_to_escrow_constraint_model(
        program_id,
        accounts,
        !creation_ix,
        args.constraint_name,
        constraint,
    )
}

fn add_collection_constraint_to_escrow_constraint_model(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: AddCollectionConstraintToEscrowConstraintModelArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    accounts_iter.next(); // skip the escrow constraint model
    accounts_iter.next(); // skip the payer
    accounts_iter.next(); // skip the update authority
    let collection_mint_info = next_account_info(accounts_iter)?;
    let collection_metadata_info = next_account_info(accounts_iter)?;
    accounts_iter.next(); // skip the system program
    let sysvar_instruction_info = next_account_info(accounts_iter)?;

    assert_owned_by(collection_mint_info, &spl_token::id())?;
    assert_owned_by(collection_metadata_info, &mpl_token_metadata::id())?;

    Metadata::from_account_info(collection_metadata_info)
        .map_err(|_| TrifleError::InvalidCollectionMetadata)?;

    let constraint = EscrowConstraint {
        constraint_type: EscrowConstraintType::Collection(*collection_mint_info.key),
        token_limit: args.token_limit,
        transfer_effects: args.transfer_effects,
    };

    // Check if the previous instruction was a creation instruction, so we don't double-charge protocol fees.
    let mut creation_ix = false;
    if load_current_index_checked(sysvar_instruction_info)? > 0 {
        let prev_ix = get_instruction_relative(-1, sysvar_instruction_info)?;
        creation_ix = is_creation_instruction(*prev_ix.data.first().unwrap_or(&255));
    }

    add_constraint_to_escrow_constraint_model(
        program_id,
        accounts,
        !creation_ix,
        args.constraint_name,
        constraint,
    )
}

fn add_tokens_constraint_to_escrow_constraint_model(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: AddTokensConstraintToEscrowConstraintModelArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    accounts_iter.next(); // skip the escrow constraint model
    accounts_iter.next(); // skip the payer
    accounts_iter.next(); // skip the update authority
    accounts_iter.next(); // skip the system program
    let sysvar_instruction_info = next_account_info(accounts_iter)?;

    let constraint = EscrowConstraint {
        constraint_type: EscrowConstraintType::tokens_from_slice(&args.tokens),
        token_limit: args.token_limit,
        transfer_effects: args.transfer_effects,
    };

    // Check if the previous instruction was a creation instruction, so we don't double-charge protocol fees.
    let mut creation_ix = false;
    if load_current_index_checked(sysvar_instruction_info)? > 0 {
        let prev_ix = get_instruction_relative(-1, sysvar_instruction_info)?;
        creation_ix = is_creation_instruction(*prev_ix.data.first().unwrap_or(&255));
    }

    add_constraint_to_escrow_constraint_model(
        program_id,
        accounts,
        !creation_ix,
        args.constraint_name,
        constraint,
    )
}

fn remove_constraint_from_escrow_constraint_model(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: RemoveConstraintFromEscrowConstraintModelArgs,
) -> ProgramResult {
    let mut accounts_iter = accounts.iter();
    let escrow_constraint_model_info = next_account_info(&mut accounts_iter)?;
    let payer_info = next_account_info(&mut accounts_iter)?;
    let update_authority_info = next_account_info(&mut accounts_iter)?;
    let system_program_info = next_account_info(&mut accounts_iter)?;

    assert_signer(payer_info)?;
    assert_signer(update_authority_info)?;
    assert_owned_by(escrow_constraint_model_info, program_id)?;
    // assert update authority matches ecm update authority;
    let mut escrow_constraint_model =
        EscrowConstraintModel::from_account_info(escrow_constraint_model_info)?;

    if escrow_constraint_model.update_authority != *update_authority_info.key {
        return Err(TrifleError::InvalidUpdateAuthority.into());
    }

    // remove the constraint by key.
    escrow_constraint_model
        .constraints
        .remove(&args.constraint_name);

    pay_royalties(
        RoyaltyInstruction::RemoveConstraint,
        &mut escrow_constraint_model,
        payer_info,
        escrow_constraint_model_info,
        system_program_info,
    )?;

    let serialized_data = escrow_constraint_model
        .try_to_vec()
        .map_err(|_| TrifleError::FailedToSerialize)?;

    // resize the account to the new size.
    resize_or_reallocate_account_raw(
        escrow_constraint_model_info,
        payer_info,
        system_program_info,
        serialized_data.len(),
    )?;

    sol_memcpy(
        &mut **escrow_constraint_model_info
            .try_borrow_mut_data()
            .map_err(|_| TrifleError::FailedToBorrowAccountData)?,
        &serialized_data,
        serialized_data.len(),
    );

    Ok(())
}

fn set_royalties(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: SetRoyaltiesArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let constraint_model_info = next_account_info(accounts_iter)?;
    let payer_info = next_account_info(accounts_iter)?;
    let _update_authority_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;
    let _sysvar_instruction_info = next_account_info(accounts_iter)?;

    let bump = assert_derivation(
        program_id,
        constraint_model_info,
        &[
            ESCROW_SEED.as_bytes(),
            payer_info.key.as_ref(),
            args.name.as_bytes(),
        ],
    )?;

    let _constraint_model_seeds = &[
        ESCROW_SEED.as_ref(),
        payer_info.key.as_ref(),
        args.name.as_ref(),
        &[bump],
    ];

    let mut constraint_model =
        EscrowConstraintModel::try_from_slice(&constraint_model_info.data.borrow())
            .map_err(|_| TrifleError::InvalidEscrowConstraintModel)?;

    // Royalties are set on a per-instruction basis, so loop through each
    // IX:Royalty pair and set the royalty in the map.
    for ix_type in args.royalties {
        constraint_model
            .royalties
            .entry(ix_type.0)
            .or_insert_with(|| ix_type.1);
    }

    // collect fees and save the model.
    pay_royalties(
        RoyaltyInstruction::TransferOut,
        &mut constraint_model,
        payer_info,
        constraint_model_info,
        system_program_info,
    )?;

    let serialized_data = constraint_model
        .try_to_vec()
        .map_err(|_| TrifleError::FailedToSerialize)?;

    resize_or_reallocate_account_raw(
        constraint_model_info,
        payer_info,
        system_program_info,
        serialized_data.len(),
    )?;

    sol_memcpy(
        &mut **constraint_model_info
            .try_borrow_mut_data()
            .map_err(|_| TrifleError::FailedToBorrowAccountData)?,
        &serialized_data,
        serialized_data.len(),
    );

    Ok(())
}

fn withdraw_royalties(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: WithdrawRoyaltiesArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let constraint_model_info = next_account_info(accounts_iter)?;
    let payer_info = next_account_info(accounts_iter)?;
    let update_authority_info = next_account_info(accounts_iter)?;
    let destination_info = next_account_info(accounts_iter)?;
    let new_dest_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;
    let _sysvar_instruction_info = next_account_info(accounts_iter)?;

    assert_signer(payer_info)?;

    let bump = assert_derivation(
        program_id,
        constraint_model_info,
        &[
            ESCROW_SEED.as_bytes(),
            update_authority_info.key.as_ref(),
            args.name.as_bytes(),
        ],
    )?;

    let constraint_model_seeds = &[
        ESCROW_SEED.as_ref(),
        update_authority_info.key.as_ref(),
        args.name.as_ref(),
        &[bump],
    ];

    let mut constraint_model =
        EscrowConstraintModel::try_from_slice(&constraint_model_info.data.borrow())
            .map_err(|_| TrifleError::InvalidEscrowConstraintModel)?;

    // Check that the payer is the update authority before paying out royalties.
    if payer_info.key == update_authority_info.key {
        // Transfer the creator royalties balance to the destination account
        // and set the balance to 0 afterwards.
        invoke_signed(
            &system_instruction::transfer(
                constraint_model_info.key,
                destination_info.key,
                constraint_model.royalty_balance,
            ),
            &[
                constraint_model_info.clone(),
                destination_info.clone(),
                constraint_model_info.clone(),
                system_program_info.clone(),
            ],
            &[constraint_model_seeds],
        )?;

        constraint_model.royalty_balance = 0;
    }

    let serialized_data = constraint_model
        .try_to_vec()
        .map_err(|_| TrifleError::FailedToSerialize)?;

    // Transfer the remaining balance to the Metaplex DAO. The untracked balance
    // (account.lamports - rent - royalty_balance) is the total collected protocol fees.
    invoke_signed(
        &system_instruction::transfer(
            constraint_model_info.key,
            new_dest_info.key,
            constraint_model_info
                .lamports()
                .checked_sub(constraint_model.royalty_balance)
                .ok_or(TrifleError::NumericalOverflow)?
                .checked_sub(Rent::get()?.minimum_balance(serialized_data.len()))
                .ok_or(TrifleError::NumericalOverflow)?,
        ),
        &[
            constraint_model_info.clone(),
            new_dest_info.clone(),
            constraint_model_info.clone(),
            system_program_info.clone(),
        ],
        &[constraint_model_seeds],
    )?;

    if payer_info.key == update_authority_info.key {
        resize_or_reallocate_account_raw(
            constraint_model_info,
            payer_info,
            system_program_info,
            serialized_data.len(),
        )?;

        sol_memcpy(
            &mut **constraint_model_info
                .try_borrow_mut_data()
                .map_err(|_| TrifleError::FailedToBorrowAccountData)?,
            &serialized_data,
            serialized_data.len(),
        );
    }

    Ok(())
}
