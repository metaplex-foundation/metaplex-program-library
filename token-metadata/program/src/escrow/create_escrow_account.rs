use crate::{
    error::MetadataError,
    escrow::{EscrowConstraintModel, TokenOwnedEscrow, ESCROW_PREFIX},
    instruction::MetadataInstruction,
    state::{Key, Metadata, TokenMetadataAccount, TokenStandard, PREFIX},
    utils::{
        assert_derivation, assert_owned_by, check_token_standard, create_or_allocate_account_raw,
    },
};
use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

pub fn create_escrow_account(
    program_id: Pubkey,
    escrow_account: Pubkey,
    metadata_account: Pubkey,
    mint_account: Pubkey,
    edition_account: Pubkey,
    payer_account: Pubkey,
    constraint_model: Option<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(escrow_account, false),
        AccountMeta::new_readonly(metadata_account, false),
        AccountMeta::new_readonly(mint_account, false),
        AccountMeta::new_readonly(edition_account, false),
        AccountMeta::new(payer_account, true),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    if let Some(constraint_model) = constraint_model {
        accounts.push(AccountMeta::new(constraint_model, false));
    }

    let data = MetadataInstruction::CreateEscrowAccount
        .try_to_vec()
        .unwrap();

    Instruction {
        program_id,
        accounts,
        data,
    }
}

pub fn process_create_escrow_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let escrow_account_info = next_account_info(account_info_iter)?;
    let metadata_account_info = next_account_info(account_info_iter)?;
    let mint_account_info = next_account_info(account_info_iter)?;
    let edition_account_info = next_account_info(account_info_iter)?;
    let payer_account_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let maybe_escrow_constraint_model_account = next_account_info(account_info_iter);

    // Owned by token-metadata program.
    assert_owned_by(metadata_account_info, program_id)?;
    let metadata: Metadata = Metadata::from_account_info(metadata_account_info)?;

    // Mint account passed in must be the mint of the metadata account passed in.
    if &metadata.mint != mint_account_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    // Only non-fungible tokens (i.e. unique) can have escrow accounts.
    assert!(
        check_token_standard(mint_account_info, Some(edition_account_info))?
            == TokenStandard::NonFungible,
    );

    // Assert that this is the canonical PDA for this mint.
    let bump_seed = assert_derivation(
        program_id,
        escrow_account_info,
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            mint_account_info.key.as_ref(),
            ESCROW_PREFIX.as_bytes(),
        ],
    )?;

    //assert_update_authority_is_correct(&metadata, update_authority_info)?;

    // Derive the seeds for PDA signing.
    let escrow_authority_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        metadata.mint.as_ref(),
        ESCROW_PREFIX.as_bytes(),
        &[bump_seed],
    ];

    // Initialize a default (empty) escrow structure.
    let mut toe = TokenOwnedEscrow {
        key: Key::TokenOwnedEscrow,
        base_token: *mint_account_info.key,
        tokens: vec![],
        delegates: vec![],
        model: None,
    };

    // If there is a constraint model and the signer is the update authority, then add the model to the TOE.
    if maybe_escrow_constraint_model_account.is_ok() {
        let escrow_constraint_model_account = maybe_escrow_constraint_model_account.unwrap();
        let mut escrow_constraint_model: EscrowConstraintModel =
            EscrowConstraintModel::from_account_info(escrow_constraint_model_account)?;
        // let escrow_constraint_model = EscrowConstraintModel::safe_deserialize(maybe_escrow_constraint_model.data)?;
        if *payer_account_info.key == metadata.update_authority {
            toe.model = Some(escrow_constraint_model_account.key.to_owned());

            escrow_constraint_model.count += 1;

            escrow_constraint_model
                .serialize(&mut *escrow_constraint_model_account.data.borrow_mut())?;
        } else {
            return Err(MetadataError::MustBeUpdateAuthToSetModel.into());
        }
    }

    // Create the account.
    create_or_allocate_account_raw(
        *program_id,
        escrow_account_info,
        system_account_info,
        payer_account_info,
        toe.len(),
        escrow_authority_seeds,
    )?;

    toe.serialize(&mut *escrow_account_info.try_borrow_mut_data()?)?;

    Ok(())
}
