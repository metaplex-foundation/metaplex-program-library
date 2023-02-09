use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use super::InstructionBuilder;
use crate::{instruction::MetadataInstruction, processor::AuthorizationData};

///# Burn Edition NFT
///
/// Burn an Edition NFT, closing its token, metadata and edition accounts, and reducing the Master Edition supply.
///
/// ### Accounts:
///
///   0. `[writable]` Print NFT Metadata Account
///   1. `[writable, signer]` Owner of Print NFT
///   2. `[writable]` Mint of Print Edition NFT
///   3. `[writable]` Mint of Master Edition NFT
///   4. `[writable]` Print Edition Token Account
///   5. `[writable]` Master Edition Token Account
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
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
}

/// Burn an asset.
///
/// # Accounts:
///
///
///   0.  `[signer, writable]` Owner of the asset
///   1.  `[optional, writable]` Collection Metadata account
///   2.  `[writable]` Item Metadata account
///   3.  `[writable]` Edition account
///   4.  `[writable]` Mint account
///   5.  `[writable]` Token account
///   6.  `[]` System program
///   7.  `[]` Instruction sysvar account
///   8.  `[]` SPL Token Program
///   9.  `[optional]` Token Authorization Rules Program
///   10. `[optional]` Token Authorization Rules account
impl InstructionBuilder for super::builders::Burn {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let mut accounts = vec![
            AccountMeta::new(self.owner, true),
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
            AccountMeta::new_readonly(self.parent_edition.unwrap_or(crate::ID), false),
            AccountMeta::new_readonly(self.parent_mint.unwrap_or(crate::ID), false),
            AccountMeta::new_readonly(self.parent_token.unwrap_or(crate::ID), false),
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

        // Optional authorization rules accounts
        if let Some(rules) = &self.authorization_rules {
            accounts.push(AccountMeta::new_readonly(mpl_token_auth_rules::ID, false));
            accounts.push(AccountMeta::new_readonly(*rules, false));
        } else {
            accounts.push(AccountMeta::new_readonly(crate::ID, false));
            accounts.push(AccountMeta::new_readonly(crate::ID, false));
        }

        Instruction {
            program_id: crate::ID,
            accounts,
            data: MetadataInstruction::Burn(self.args.clone())
                .try_to_vec()
                .unwrap(),
        }
    }
}
