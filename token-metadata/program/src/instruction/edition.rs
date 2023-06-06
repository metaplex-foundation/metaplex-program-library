use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use crate::{
    instruction::MetadataInstruction,
    state::{EDITION, EDITION_MARKER_BIT_SIZE, PREFIX},
};

/// Converts a master edition v1 to v2
#[allow(clippy::too_many_arguments)]
pub fn convert_master_edition_v1_to_v2(
    program_id: Pubkey,
    master_edition: Pubkey,
    one_time_auth: Pubkey,
    printing_mint: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(master_edition, false),
            AccountMeta::new(one_time_auth, false),
            AccountMeta::new(printing_mint, false),
        ],
        data: MetadataInstruction::ConvertMasterEditionV1ToV2
            .try_to_vec()
            .unwrap(),
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct CreateMasterEditionArgs {
    /// If set, means that no more than this number of editions can ever be minted. This is immutable.
    pub max_supply: Option<u64>,
}

/// creates a create_master_edition instruction
#[allow(clippy::too_many_arguments)]
pub fn create_master_edition_v3(
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
        AccountMeta::new(metadata, false),
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::CreateMasterEditionV3(CreateMasterEditionArgs { max_supply })
            .try_to_vec()
            .unwrap(),
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct MintNewEditionFromMasterEditionViaTokenArgs {
    pub edition: u64,
}

/// creates a mint_new_edition_from_master_edition instruction
#[allow(clippy::too_many_arguments)]
pub fn mint_new_edition_from_master_edition_via_token(
    program_id: Pubkey,
    new_metadata: Pubkey,
    new_edition: Pubkey,
    master_edition: Pubkey,
    new_mint: Pubkey,
    new_mint_authority: Pubkey,
    payer: Pubkey,
    token_account_owner: Pubkey,
    token_account: Pubkey,
    new_metadata_update_authority: Pubkey,
    metadata: Pubkey,
    metadata_mint: Pubkey,
    edition: u64,
) -> Instruction {
    let edition_number = edition.checked_div(EDITION_MARKER_BIT_SIZE).unwrap();
    let as_string = edition_number.to_string();
    let (edition_mark_pda, _) = Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            metadata_mint.as_ref(),
            EDITION.as_bytes(),
            as_string.as_bytes(),
        ],
        &program_id,
    );

    let accounts = vec![
        AccountMeta::new(new_metadata, false),
        AccountMeta::new(new_edition, false),
        AccountMeta::new(master_edition, false),
        AccountMeta::new(new_mint, false),
        AccountMeta::new(edition_mark_pda, false),
        AccountMeta::new_readonly(new_mint_authority, true),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(token_account_owner, true),
        AccountMeta::new_readonly(token_account, false),
        AccountMeta::new_readonly(new_metadata_update_authority, false),
        AccountMeta::new_readonly(metadata, false),
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::MintNewEditionFromMasterEditionViaToken(
            MintNewEditionFromMasterEditionViaTokenArgs { edition },
        )
        .try_to_vec()
        .unwrap(),
    }
}
