use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

use crate::instruction::MetadataInstruction;

pub fn close_escrow_account(
    program_id: Pubkey,
    escrow_account: Pubkey,
    metadata_account: Pubkey,
    mint_account: Pubkey,
    edition_account: Pubkey,
    payer_account: Pubkey,
    token_account: Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(escrow_account, false),
        AccountMeta::new(metadata_account, false),
        AccountMeta::new_readonly(mint_account, false),
        AccountMeta::new_readonly(token_account, false),
        AccountMeta::new_readonly(edition_account, false),
        AccountMeta::new(payer_account, true),
        AccountMeta::new_readonly(system_program::ID, false),
        AccountMeta::new_readonly(sysvar::instructions::ID, false),
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

pub fn create_escrow_account(
    program_id: Pubkey,
    escrow_account: Pubkey,
    metadata_account: Pubkey,
    mint_account: Pubkey,
    token_account: Pubkey,
    edition_account: Pubkey,
    payer_account: Pubkey,
    authority: Option<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(escrow_account, false),
        AccountMeta::new(metadata_account, false),
        AccountMeta::new_readonly(mint_account, false),
        AccountMeta::new_readonly(token_account, false),
        AccountMeta::new_readonly(edition_account, false),
        AccountMeta::new(payer_account, true),
        AccountMeta::new_readonly(solana_program::system_program::ID, false),
        AccountMeta::new_readonly(sysvar::instructions::ID, false),
    ];

    if let Some(authority) = authority {
        accounts.push(AccountMeta::new_readonly(authority, true));
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

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct TransferOutOfEscrowArgs {
    pub amount: u64,
}

pub fn transfer_out_of_escrow(
    program_id: Pubkey,
    escrow: Pubkey,
    metadata: Pubkey,
    payer: Pubkey,
    attribute_mint: Pubkey,
    attribute_src: Pubkey,
    attribute_dst: Pubkey,
    escrow_mint: Pubkey,
    escrow_account: Pubkey,
    authority: Option<Pubkey>,
    amount: u64,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new_readonly(escrow, false),
        AccountMeta::new(metadata, false),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(attribute_mint, false),
        AccountMeta::new(attribute_src, false),
        AccountMeta::new(attribute_dst, false),
        AccountMeta::new_readonly(escrow_mint, false),
        AccountMeta::new_readonly(escrow_account, false),
        AccountMeta::new_readonly(solana_program::system_program::ID, false),
        AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new_readonly(sysvar::instructions::ID, false),
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
