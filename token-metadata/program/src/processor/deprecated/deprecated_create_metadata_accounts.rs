use borsh::{BorshDeserialize, BorshSerialize};
pub use instruction::*;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};

use crate::{
    instruction::MetadataInstruction,
    state::{Creator, Data, DataV2},
    utils::{process_create_metadata_accounts_logic, CreateMetadataAccountsLogicArgs},
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
    /// Args for create call
    pub struct CreateMetadataAccountArgs {
        /// Note that unique metadatas are disabled for now.
        pub data: Data,
        /// Whether you want your metadata to be updateable in the future.
        pub is_mutable: bool,
    }

    /// Creates an CreateMetadataAccounts instruction
    /// #[deprecated(since="1.1.0", note="please use `create_metadata_accounts_v3` instead")]
    #[allow(clippy::too_many_arguments)]
    pub fn create_metadata_accounts(
        program_id: Pubkey,
        metadata_account: Pubkey,
        mint: Pubkey,
        mint_authority: Pubkey,
        payer: Pubkey,
        update_authority: Pubkey,
        name: String,
        symbol: String,
        uri: String,
        creators: Option<Vec<Creator>>,
        seller_fee_basis_points: u16,
        update_authority_is_signer: bool,
        is_mutable: bool,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(metadata_account, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(mint_authority, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(update_authority, update_authority_is_signer),
                AccountMeta::new_readonly(solana_program::system_program::id(), false),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
            ],
            data: MetadataInstruction::CreateMetadataAccount(CreateMetadataAccountArgs {
                data: Data {
                    name,
                    symbol,
                    uri,
                    seller_fee_basis_points,
                    creators,
                },
                is_mutable,
            })
            .try_to_vec()
            .unwrap(),
        }
    }
}

pub fn process_deprecated_create_metadata_accounts<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    data: Data,
    is_mutable: bool,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let metadata_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let payer_account_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;

    process_create_metadata_accounts_logic(
        program_id,
        CreateMetadataAccountsLogicArgs {
            metadata_account_info,
            mint_info,
            mint_authority_info,
            payer_account_info,
            update_authority_info,
            system_account_info,
        },
        DataV2 {
            name: data.name,
            uri: data.uri,
            symbol: data.symbol,
            creators: data.creators,
            seller_fee_basis_points: data.seller_fee_basis_points,
            collection: None,
            uses: None,
        },
        false,
        is_mutable,
        false,
        false,
        None, // Does not support collection parents.
    )
}
