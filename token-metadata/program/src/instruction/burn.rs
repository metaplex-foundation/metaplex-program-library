use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::{instruction::MetadataInstruction, processor::AuthorizationData};

///# Burn Edition NFT
///
/// Burn an Edition NFT, closing its token, metadata and edition accounts, and reducing the Master Edition supply.
///
/// ### Accounts:
///
///   0. [writable] Print NFT Metadata Account
///   1. [writable, signer]</code> Owner of Print NFT
///   2. [writable] Mint of Print Edition NFT
///   3. [writable] Mint of Master Edition NFT
///   4. [writable] Print Edition Token Account
///   5. [writable] Master Edition Token Account
///   6. [writable] Master Edition PDA Account
///   7. [writable] Print Edition PDA Account
///   8. [writable] Edition Marker PDA Account
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
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
}
