use borsh::BorshSerialize;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::instruction::{MetadataInstruction, SetCollectionSizeArgs};

pub fn bubblegum_set_collection_size(
    program_id: Pubkey,
    metadata_account: Pubkey,
    update_authority: Pubkey,
    mint: Pubkey,
    bubblegum_signer: Pubkey,
    collection_authority_record: Option<Pubkey>,
    size: u64,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(metadata_account, false),
        AccountMeta::new_readonly(update_authority, true),
        AccountMeta::new_readonly(mint, false),
        AccountMeta::new_readonly(bubblegum_signer, true),
    ];

    if let Some(record) = collection_authority_record {
        accounts.push(AccountMeta::new_readonly(record, false));
    }

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::BubblegumSetCollectionSize(SetCollectionSizeArgs { size })
            .try_to_vec()
            .unwrap(),
    }
}
