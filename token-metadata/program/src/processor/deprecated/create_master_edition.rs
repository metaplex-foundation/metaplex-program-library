use borsh::BorshSerialize;
pub use instruction::*;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};

use crate::{instruction::MetadataInstruction, processor::CreateMasterEditionArgs};

mod instruction {
    use super::*;

    /// creates a create_master_edition instruction
    #[allow(clippy::too_many_arguments)]
    /// [deprecated(since="1.1.0", note="please use `create_master_edition_v3` instead")]
    pub fn create_master_edition(
        program_id: Pubkey,
        edition: Pubkey,
        mint: Pubkey,
        update_authority: Pubkey,
        mint_authority: Pubkey,
        metadata: Pubkey,
        payer: Pubkey,
        max_supply: Option<u64>,
    ) -> Instruction {
        let accounts = vec![
            AccountMeta::new(edition, false),
            AccountMeta::new(mint, false),
            AccountMeta::new_readonly(update_authority, true),
            AccountMeta::new_readonly(mint_authority, true),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(metadata, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ];

        Instruction {
            program_id,
            accounts,
            data: MetadataInstruction::CreateMasterEdition(CreateMasterEditionArgs { max_supply })
                .try_to_vec()
                .unwrap(),
        }
    }
}
