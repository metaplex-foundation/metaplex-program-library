use crate::{
    error::MetadataError,
    instruction::MetadataInstruction,
    state::{
        Metadata, TokenMetadataAccount, TokenOwnedEscrow, TokenStandard, ESCROW_PREFIX, PREFIX,
    },
    utils::{assert_derivation, assert_owned_by, check_token_standard, close_account_raw},
};
use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

pub fn close_escrow_account(
    program_id: Pubkey,
    escrow_account: Pubkey,
    metadata_account: Pubkey,
    mint_account: Pubkey,
    edition_account: Pubkey,
    payer_account: Pubkey,
    // system_account: Pubkey,
    // rent: Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(escrow_account, false),
        AccountMeta::new_readonly(metadata_account, false),
        AccountMeta::new_readonly(mint_account, false),
        AccountMeta::new_readonly(edition_account, false),
        AccountMeta::new(payer_account, true),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
    ];
    let data = MetadataInstruction::CloseEscrowAccount
        .try_to_vec()
        .unwrap();

    Instruction {
        program_id,
        accounts,
        data,
    }
}

pub fn process_close_escrow_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let escrow_account_info = next_account_info(account_info_iter)?;
    let metadata_account_info = next_account_info(account_info_iter)?;
    let mint_account_info = next_account_info(account_info_iter)?;
    let edition_account_info = next_account_info(account_info_iter)?;
    let payer_account_info = next_account_info(account_info_iter)?;
    let _system_account_info = next_account_info(account_info_iter)?;
    let _rent_info = next_account_info(account_info_iter)?;

    // Owned by token-metadata program.
    assert_owned_by(metadata_account_info, program_id)?;
    let metadata: Metadata = Metadata::from_account_info(metadata_account_info)?;

    // Mint account passed in must be the mint of the metadata account passed in.
    if &metadata.mint != mint_account_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    assert!(
        check_token_standard(mint_account_info, Some(edition_account_info))?
            == TokenStandard::NonFungible,
    );

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

    let _escrow_authority_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        metadata.mint.as_ref(),
        ESCROW_PREFIX.as_bytes(),
        &[bump_seed],
    ];

    let toe: TokenOwnedEscrow = TokenOwnedEscrow::from_account_info(escrow_account_info)?;
    assert!(!toe.tokens.is_empty());

    // Close the account.
    close_account_raw(payer_account_info, escrow_account_info)?;

    Ok(())
}
