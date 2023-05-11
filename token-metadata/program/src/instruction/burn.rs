use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use super::InstructionBuilder;
use crate::instruction::MetadataInstruction;

///# Burn Edition NFT
///
/// Burn an Edition NFT, closing its token, metadata and edition accounts, and reducing the Master Edition supply.
///
/// ### Accounts:
///
///   0. `[writable]` Print NFT Metadata Account
///   1. `[writable, signer]` Owner of Print NFT
///   2. `[writable]` Mint of Print Edition NFT
///   3. `[]` Mint of Master Edition NFT
///   4. `[writable]` Print Edition Token Account
///   5. `[]` Master Edition Token Account
///   6. `[writable]` Master Edition PDA Account
///   7. `[writable]` Print Edition PDA Account
///   8. `[writable]` Edition Marker PDA Account
///   9. [] SPL Token program.
pub fn burn_edition_nft(
    program_id: Pubkey,
    metadata: Pubkey,
    owner: Pubkey,
    print_edition_mint: Pubkey,
    master_edition_mint: Pubkey,
    print_edition_token: Pubkey,
    master_edition_token: Pubkey,
    master_edition: Pubkey,
    print_edition: Pubkey,
    edition_marker: Pubkey,
    spl_token: Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(metadata, false),
        AccountMeta::new(owner, true),
        AccountMeta::new(print_edition_mint, false),
        AccountMeta::new_readonly(master_edition_mint, false),
        AccountMeta::new(print_edition_token, false),
        AccountMeta::new_readonly(master_edition_token, false),
        AccountMeta::new(master_edition, false),
        AccountMeta::new(print_edition, false),
        AccountMeta::new(edition_marker, false),
        AccountMeta::new_readonly(spl_token, false),
    ];

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::BurnEditionNft.try_to_vec().unwrap(),
    }
}

///# Burn NFT
///
/// Burn an NFT, closing its token, metadata and edition accounts.
///
/// 0. `[writable]` NFT metadata
/// 1. `[writable, signer]` Owner of NFT
/// 2. `[writable]` Mint of NFT
/// 3. `[writable]` NFT token account
/// 4. `[writable]` NFT edition account
/// 5. `[]` SPL Token program.
/// 6. Optional `[writable]` Collection metadata account
pub fn burn_nft(
    program_id: Pubkey,
    metadata: Pubkey,
    owner: Pubkey,
    mint: Pubkey,
    token: Pubkey,
    edition: Pubkey,
    spl_token: Pubkey,
    collection_metadata: Option<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(metadata, false),
        AccountMeta::new(owner, true),
        AccountMeta::new(mint, false),
        AccountMeta::new(token, false),
        AccountMeta::new(edition, false),
        AccountMeta::new_readonly(spl_token, false),
    ];

    if let Some(collection_metadata) = collection_metadata {
        accounts.push(AccountMeta::new(collection_metadata, false));
    }

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::BurnNft.try_to_vec().unwrap(),
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum BurnArgs {
    V1 {
        /// The amount of the token to burn
        amount: u64,
    },
}

/// Burn an asset.
///
/// # Accounts:
///
///
///   0.   `[signer, writable]` Asset owner or Utility delegate
///   1.   `[optional, writable]` Metadata of the Collection
///   2.   `[writable]` Metadata (pda of ['metadata', program id, mint id])
///   3.   `[optional, writable]` Edition of the asset
///   4.   `[writable]` Mint of token account
///   5.   `[writable]` Token account to close
///   6.   `[optional, writable]` Master edition token account
///   7.   `[optional]` Master edition mint of the asset
///   8.   `[optional]` Master edition token account
///   9.   `[optional, writable]` Edition marker account
///  10.   `[optional, writable]` Token record account
///  11.   `[]` System program
///  12.   `[]` Instruction sysvar account
///  13.   `[]` SPL Token Program
impl InstructionBuilder for super::builders::Burn {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let accounts = vec![
            AccountMeta::new(self.authority, true),
            if let Some(collection_metadata) = self.collection_metadata {
                AccountMeta::new(collection_metadata, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            AccountMeta::new(self.metadata, false),
            if let Some(edition) = self.edition {
                AccountMeta::new(edition, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            AccountMeta::new(self.mint, false),
            AccountMeta::new(self.token, false),
            if let Some(master_edition) = self.master_edition {
                AccountMeta::new(master_edition, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            AccountMeta::new_readonly(self.master_edition_mint.unwrap_or(crate::ID), false),
            AccountMeta::new_readonly(self.master_edition_token.unwrap_or(crate::ID), false),
            if let Some(edition_marker) = self.edition_marker {
                AccountMeta::new(edition_marker, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            if let Some(token_record) = self.token_record {
                AccountMeta::new(token_record, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            AccountMeta::new_readonly(self.system_program, false),
            AccountMeta::new_readonly(self.sysvar_instructions, false),
            AccountMeta::new_readonly(self.spl_token_program, false),
        ];

        Instruction {
            program_id: crate::ID,
            accounts,
            data: MetadataInstruction::Burn(self.args.clone())
                .try_to_vec()
                .unwrap(),
        }
    }
}
