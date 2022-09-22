use crate::{
    error::MetadataError,
    instruction::MetadataInstruction,
    state::{Metadata, TokenMetadataAccount, TokenOwnedEscrow, ESCROW_PREFIX, PREFIX},
    utils::{assert_derivation, assert_owned_by, assert_signer},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
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
    authority: Option<Pubkey>,
    amount: u64,
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

    if let Some(authority) = authority {
        accounts.push(AccountMeta::new_readonly(authority, true));
    }

    let data = MetadataInstruction::TransferOutOfEscrow(TransferOutOfEscrowArgs { amount })
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

    let is_using_authority = account_info_iter.len() == 13;

    let maybe_authority_info: Option<&AccountInfo> = if is_using_authority {
        Some(next_account_info(account_info_iter)?)
    } else {
        None
    };

    let toe = TokenOwnedEscrow::from_account_info(escrow_info).unwrap();

    // Owned by token-metadata program.
    assert_owned_by(attribute_metadata_info, program_id)?;
    let _attribute_metadata: Metadata = Metadata::from_account_info(attribute_metadata_info)?;
    let authority_primitive: Vec<u8> = toe.authority.try_to_vec().unwrap();

    let bump_seed = assert_derivation(
        program_id,
        escrow_info,
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            escrow_mint_info.key.as_ref(),
            authority_primitive.as_ref(),
            ESCROW_PREFIX.as_bytes(),
        ],
    )?;

    // Derive the seeds for PDA signing.
    let escrow_authority_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        escrow_mint_info.key.as_ref(),
        authority_primitive.as_ref(),
        ESCROW_PREFIX.as_bytes(),
        &[bump_seed],
    ];

    assert_signer(payer_info)?;

    // Allocate the escrow accounts new ATA.
    let create_escrow_ata_ix =
        spl_associated_token_account::instruction::create_associated_token_account(
            payer_info.key,
            payer_info.key,
            attribute_mint_info.key,
        );

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

    // Deserialize the token accounts and perform checks.
    let attribute_src = spl_token::state::Account::unpack(&attribute_src_info.data.borrow())?;
    assert!(attribute_src.mint == *attribute_mint_info.key);
    assert!(attribute_src.delegate.is_none());
    assert!(attribute_src.amount >= args.amount);
    let attribute_dst = spl_token::state::Account::unpack(&attribute_dst_info.data.borrow())?;
    assert!(attribute_dst.mint == *attribute_mint_info.key);
    assert!(attribute_dst.delegate.is_none());
    let escrow_account = spl_token::state::Account::unpack(&escrow_account_info.data.borrow())?;

    assert!(attribute_dst.owner == escrow_account.owner);

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

    Ok(())
}
