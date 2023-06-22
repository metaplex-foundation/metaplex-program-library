use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::state::fee::FEE_AUTHORITY;

use super::*;

pub fn collect_fees(recipient: Pubkey, fee_accounts: Vec<Pubkey>) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(FEE_AUTHORITY, true),
        AccountMeta::new(recipient, false),
    ];

    for fee_account in fee_accounts {
        accounts.push(AccountMeta::new(fee_account, false));
    }
    Instruction {
        program_id: crate::ID,
        accounts,
        data: MetadataInstruction::Collect.try_to_vec().unwrap(),
    }
}
