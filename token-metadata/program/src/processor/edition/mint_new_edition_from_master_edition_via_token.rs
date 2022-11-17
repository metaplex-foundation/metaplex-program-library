use borsh::{BorshDeserialize, BorshSerialize};
pub use instruction::*;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::{
    instruction::MetadataInstruction,
    state::{EDITION, EDITION_MARKER_BIT_SIZE, PREFIX},
    utils::{
        process_mint_new_edition_from_master_edition_via_token_logic,
        MintNewEditionFromMasterEditionViaTokenLogicArgs,
    },
};

mod instruction {
    #[cfg(feature = "serde-feature")]
    use {
        serde::{Deserialize, Serialize},
        serde_with::{As, DisplayFromStr},
    };

    use super::*;

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
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
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
}

pub fn process_mint_new_edition_from_master_edition_via_token<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    edition: u64,
    ignore_owner_signer: bool,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let new_metadata_account_info = next_account_info(account_info_iter)?;
    let new_edition_account_info = next_account_info(account_info_iter)?;
    let master_edition_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let edition_marker_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let payer_account_info = next_account_info(account_info_iter)?;
    let owner_account_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let master_metadata_account_info = next_account_info(account_info_iter)?;
    let token_program_account_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;

    process_mint_new_edition_from_master_edition_via_token_logic(
        program_id,
        MintNewEditionFromMasterEditionViaTokenLogicArgs {
            new_metadata_account_info,
            new_edition_account_info,
            master_edition_account_info,
            mint_info,
            edition_marker_info,
            mint_authority_info,
            payer_account_info,
            owner_account_info,
            token_account_info,
            update_authority_info,
            master_metadata_account_info,
            token_program_account_info,
            system_account_info,
        },
        edition,
        ignore_owner_signer,
    )
}
