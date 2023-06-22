use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use crate::instruction::MetadataInstruction;

///# Approve Collection Authority
///
///Approve another account to verify NFTs belonging to a collection, [verify_collection] on the collection NFT
///
///### Accounts:
///   0. `[writable]` Collection Authority Record PDA
///   1. `[signer]` Update Authority of Collection NFT
///   2. `[signer]` Payer
///   3. `[]` A Collection Authority
///   4. `[]` Collection Metadata account
///   5. `[]` Mint of Collection Metadata
///   6. `[]` Token program
///   7. `[]` System program
///   8. Optional `[]` Rent info
#[allow(clippy::too_many_arguments)]
pub fn approve_collection_authority(
    program_id: Pubkey,
    collection_authority_record: Pubkey,
    new_collection_authority: Pubkey,
    update_authority: Pubkey,
    payer: Pubkey,
    metadata: Pubkey,
    mint: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(collection_authority_record, false),
            AccountMeta::new_readonly(new_collection_authority, false),
            AccountMeta::new(update_authority, true),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(metadata, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: MetadataInstruction::ApproveCollectionAuthority
            .try_to_vec()
            .unwrap(),
    }
}

//# Revoke Collection Authority
///
///Revoke account to call [verify_collection] on this NFT
///
///### Accounts:
///
///   0. `[writable]` Collection Authority Record PDA
///   1. `[writable]` The Authority that was delegated to
///   2. `[signer]` The Original Update Authority or Delegated Authority
///   2. `[]` Metadata account
///   3. `[]` Mint of Metadata
#[allow(clippy::too_many_arguments)]
pub fn revoke_collection_authority(
    program_id: Pubkey,
    collection_authority_record: Pubkey,
    delegate_authority: Pubkey,
    revoke_authority: Pubkey,
    metadata: Pubkey,
    mint: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(collection_authority_record, false),
            AccountMeta::new_readonly(delegate_authority, false),
            AccountMeta::new(revoke_authority, true),
            AccountMeta::new_readonly(metadata, false),
            AccountMeta::new_readonly(mint, false),
        ],
        data: MetadataInstruction::RevokeCollectionAuthority
            .try_to_vec()
            .unwrap(),
    }
}

//# Set And Verify Collection
///
///Allows the same Update Authority (Or Delegated Authority) on an NFT and Collection to
/// perform update_metadata_accounts_v2 with collection and [verify_collection] on the
/// NFT/Collection in one instruction.
///
/// ### Accounts:
///
///   0. `[writable]` Metadata account
///   1. `[signer]` Collection Update authority
///   2. `[signer]` payer
///   3. `[] Update Authority of Collection NFT and NFT
///   3. `[]` Mint of the Collection
///   4. `[]` Metadata Account of the Collection
///   5. `[]` MasterEdition2 Account of the Collection Token
#[allow(clippy::too_many_arguments)]
pub fn set_and_verify_collection(
    program_id: Pubkey,
    metadata: Pubkey,
    collection_authority: Pubkey,
    payer: Pubkey,
    update_authority: Pubkey,
    collection_mint: Pubkey,
    collection: Pubkey,
    collection_master_edition_account: Pubkey,
    collection_authority_record: Option<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(metadata, false),
        AccountMeta::new(collection_authority, true),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(update_authority, false),
        AccountMeta::new_readonly(collection_mint, false),
        AccountMeta::new_readonly(collection, false),
        AccountMeta::new_readonly(collection_master_edition_account, false),
    ];

    if let Some(collection_authority_record) = collection_authority_record {
        accounts.push(AccountMeta::new_readonly(
            collection_authority_record,
            false,
        ));
    }

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::SetAndVerifyCollection
            .try_to_vec()
            .unwrap(),
    }
}

//# Set And Verify Collection V2 -- Supports v1.3 Collection Details
///
///Allows the same Update Authority (Or Delegated Authority) on an NFT and Collection to perform update_metadata_accounts_v2 with collection and [verify_collection] on the NFT/Collection in one instruction
///
/// ### Accounts:
///
///   0. `[writable]` Metadata account
///   1. `[signer]` Collection Update authority
///   2. `[signer]` payer
///   3. `[] Update Authority of Collection NFT and NFT
///   3. `[]` Mint of the Collection
///   4. `[writable]` Metadata Account of the Collection
///   5. `[]` MasterEdition2 Account of the Collection Token
#[allow(clippy::too_many_arguments)]
pub fn set_and_verify_sized_collection_item(
    program_id: Pubkey,
    metadata: Pubkey,
    collection_authority: Pubkey,
    payer: Pubkey,
    update_authority: Pubkey,
    collection_mint: Pubkey,
    collection: Pubkey,
    collection_master_edition_account: Pubkey,
    collection_authority_record: Option<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(metadata, false),
        AccountMeta::new(collection_authority, true),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(update_authority, false),
        AccountMeta::new_readonly(collection_mint, false),
        AccountMeta::new(collection, false),
        AccountMeta::new_readonly(collection_master_edition_account, false),
    ];

    if let Some(collection_authority_record) = collection_authority_record {
        accounts.push(AccountMeta::new_readonly(
            collection_authority_record,
            false,
        ));
    }

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::SetAndVerifySizedCollectionItem
            .try_to_vec()
            .unwrap(),
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct SetCollectionSizeArgs {
    pub size: u64,
}

pub fn set_collection_size(
    program_id: Pubkey,
    metadata_account: Pubkey,
    update_authority: Pubkey,
    mint: Pubkey,
    collection_authority_record: Option<Pubkey>,
    size: u64,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(metadata_account, false),
        AccountMeta::new_readonly(update_authority, true),
        AccountMeta::new_readonly(mint, false),
    ];

    if let Some(record) = collection_authority_record {
        accounts.push(AccountMeta::new_readonly(record, false));
    }

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::SetCollectionSize(SetCollectionSizeArgs { size })
            .try_to_vec()
            .unwrap(),
    }
}

/// # Unverify Collection
///
/// If a MetadataAccount Has a Collection allow an Authority of the Collection to unverify an NFT in a Collection
///
/// ### Accounts:
///
///   0. `[writable]` Metadata account
///   1. `[signer]` Collection Authority
///   2. `[signer]` payer
///   3. `[]` Mint of the Collection
///   4. `[]` Metadata Account of the Collection
///   5. `[]` MasterEdition2 Account of the Collection Token
#[allow(clippy::too_many_arguments)]
pub fn unverify_collection(
    program_id: Pubkey,
    metadata: Pubkey,
    collection_authority: Pubkey,
    collection_mint: Pubkey,
    collection: Pubkey,
    collection_master_edition_account: Pubkey,
    collection_authority_record: Option<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(metadata, false),
        AccountMeta::new(collection_authority, true),
        AccountMeta::new_readonly(collection_mint, false),
        AccountMeta::new_readonly(collection, false),
        AccountMeta::new_readonly(collection_master_edition_account, false),
    ];

    if let Some(collection_authority_record) = collection_authority_record {
        accounts.push(AccountMeta::new_readonly(
            collection_authority_record,
            false,
        ));
    }

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::UnverifyCollection
            .try_to_vec()
            .unwrap(),
    }
}

/// # Unverify Collection V2 -- Supports v1.3 Collection Details
///
/// If a MetadataAccount Has a Collection allow an Authority of the Collection to unverify an NFT in a Collection
///
/// ### Accounts:
///
///   0. `[writable]` Metadata account
///   1. `[signer]` Collection Authority
///   2. `[signer]` payer
///   3. `[]` Mint of the Collection
///   4. `[writable]` Metadata Account of the Collection
///   5. `[]` MasterEdition2 Account of the Collection Token
#[allow(clippy::too_many_arguments)]
pub fn unverify_sized_collection_item(
    program_id: Pubkey,
    metadata: Pubkey,
    collection_authority: Pubkey,
    payer: Pubkey,
    collection_mint: Pubkey,
    collection: Pubkey,
    collection_master_edition_account: Pubkey,
    collection_authority_record: Option<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(metadata, false),
        AccountMeta::new_readonly(collection_authority, true),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(collection_mint, false),
        AccountMeta::new(collection, false),
        AccountMeta::new_readonly(collection_master_edition_account, false),
    ];

    if let Some(collection_authority_record) = collection_authority_record {
        accounts.push(AccountMeta::new_readonly(
            collection_authority_record,
            false,
        ));
    }

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::UnverifySizedCollectionItem
            .try_to_vec()
            .unwrap(),
    }
}

/// # Verify Collection
///
/// If a MetadataAccount Has a Collection allow the UpdateAuthority of the Collection to Verify the NFT Belongs in the Collection
///
/// ### Accounts:
///
///   0. `[writable]` Metadata account
///   1. `[signer]` Collection Update authority
///   2. `[signer]` payer
///   3. `[]` Mint of the Collection
///   4. `[]` Metadata Account of the Collection
///   5. `[]` MasterEdition2 Account of the Collection Token
#[allow(clippy::too_many_arguments)]
pub fn verify_collection(
    program_id: Pubkey,
    metadata: Pubkey,
    collection_authority: Pubkey,
    payer: Pubkey,
    collection_mint: Pubkey,
    collection: Pubkey,
    collection_master_edition_account: Pubkey,
    collection_authority_record: Option<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(metadata, false),
        AccountMeta::new(collection_authority, true),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(collection_mint, false),
        AccountMeta::new_readonly(collection, false),
        AccountMeta::new_readonly(collection_master_edition_account, false),
    ];

    if let Some(collection_authority_record) = collection_authority_record {
        accounts.push(AccountMeta::new_readonly(
            collection_authority_record,
            false,
        ));
    }

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::VerifyCollection.try_to_vec().unwrap(),
    }
}

/// # Verify Collection V2 -- Supports v1.3 Collection Details
///
/// If a MetadataAccount Has a Collection allow the UpdateAuthority of the Collection to Verify the NFT Belongs in the Collection
///
/// ### Accounts:
///
///   0. `[writable]` Metadata account
///   1. `[signer]` Collection Update authority
///   2. `[signer]` payer
///   3. `[]` Mint of the Collection
///   4. `[writable]` Metadata Account of the Collection
///   5. `[]` MasterEdition2 Account of the Collection Token
#[allow(clippy::too_many_arguments)]
pub fn verify_sized_collection_item(
    program_id: Pubkey,
    metadata: Pubkey,
    collection_authority: Pubkey,
    payer: Pubkey,
    collection_mint: Pubkey,
    collection: Pubkey,
    collection_master_edition_account: Pubkey,
    collection_authority_record: Option<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(metadata, false),
        AccountMeta::new_readonly(collection_authority, true),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(collection_mint, false),
        AccountMeta::new(collection, false),
        AccountMeta::new_readonly(collection_master_edition_account, false),
    ];

    if let Some(record) = collection_authority_record {
        accounts.push(AccountMeta::new_readonly(record, false));
    }

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::VerifySizedCollectionItem
            .try_to_vec()
            .unwrap(),
    }
}
