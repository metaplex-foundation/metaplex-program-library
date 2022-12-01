use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};
#[cfg(feature = "serde-feature")]
use {
    serde::{Deserialize, Serialize},
    serde_with::{As, DisplayFromStr},
};

use crate::{
    instruction::MetadataInstruction,
    state::{AssetData, Collection, CollectionDetails, Creator, DataV2, Uses},
};

//----------------------+
// Instruction args     |
//----------------------+

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
/// Args for create call
pub struct CreateMetadataAccountArgsV3 {
    /// Note that unique metadatas are disabled for now.
    pub data: DataV2,
    /// Whether you want your metadata to be updateable in the future.
    pub is_mutable: bool,
    /// If this is a collection parent NFT.
    pub collection_details: Option<CollectionDetails>,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum MintArgs {
    V1(AssetData),
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum TransferArgs {
    V1 {
        authorization_payload: Option<Vec<u8>>,
    },
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum UpdateArgs {
    V1 {
        data: Option<AssetData>,
        update_authority: Option<Pubkey>,
    },
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
/// Args for update call
pub struct UpdateMetadataAccountArgsV2 {
    pub data: Option<DataV2>,
    #[cfg_attr(
        feature = "serde-feature",
        serde(with = "As::<Option<DisplayFromStr>>")
    )]
    pub update_authority: Option<Pubkey>,
    pub primary_sale_happened: Option<bool>,
    pub is_mutable: Option<bool>,
}

//----------------------+
// Instruction builders |
//----------------------+

///# Create Metadata Accounts V3 -- Supports v1.3 Collection Details
///
///Create a new Metadata Account
///
/// ### Accounts:
///
///   0. `[writable]` Metadata account
///   1. `[]` Mint account
///   2. `[signer]` Mint authority
///   3. `[signer]` payer
///   4. `[signer]` Update authority
///   5. `[]` System program
///   6. Optional `[]` Rent sysvar
///
/// Creates an CreateMetadataAccounts instruction
#[allow(clippy::too_many_arguments)]
pub fn create_metadata_accounts_v3(
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
    collection: Option<Collection>,
    uses: Option<Uses>,
    collection_details: Option<CollectionDetails>,
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
        ],
        data: MetadataInstruction::CreateMetadataAccountV3(CreateMetadataAccountArgsV3 {
            data: DataV2 {
                name,
                symbol,
                uri,
                seller_fee_basis_points,
                creators,
                collection,
                uses,
            },
            is_mutable,
            collection_details,
        })
        .try_to_vec()
        .unwrap(),
    }
}

/// puff metadata account instruction
pub fn puff_metadata_account(program_id: Pubkey, metadata_account: Pubkey) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![AccountMeta::new(metadata_account, false)],
        data: MetadataInstruction::PuffMetadata.try_to_vec().unwrap(),
    }
}

/// Remove Creator Verificaton
#[allow(clippy::too_many_arguments)]
pub fn remove_creator_verification(
    program_id: Pubkey,
    metadata: Pubkey,
    creator: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(metadata, false),
            AccountMeta::new_readonly(creator, true),
        ],
        data: MetadataInstruction::RemoveCreatorVerification
            .try_to_vec()
            .unwrap(),
    }
}

pub fn set_token_standard(
    program_id: Pubkey,
    metadata_account: Pubkey,
    update_authority: Pubkey,
    mint_account: Pubkey,
    edition_account: Option<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(metadata_account, false),
        AccountMeta::new(update_authority, true),
        AccountMeta::new_readonly(mint_account, false),
    ];
    let data = MetadataInstruction::SetTokenStandard.try_to_vec().unwrap();

    if let Some(edition_account) = edition_account {
        accounts.push(AccountMeta::new_readonly(edition_account, false));
    }

    Instruction {
        program_id,
        accounts,
        data,
    }
}

/// Sign Metadata
#[allow(clippy::too_many_arguments)]
pub fn sign_metadata(program_id: Pubkey, metadata: Pubkey, creator: Pubkey) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(metadata, false),
            AccountMeta::new_readonly(creator, true),
        ],
        data: MetadataInstruction::SignMetadata.try_to_vec().unwrap(),
    }
}

// update metadata account v2 instruction
pub fn update_metadata_accounts_v2(
    program_id: Pubkey,
    metadata_account: Pubkey,
    update_authority: Pubkey,
    new_update_authority: Option<Pubkey>,
    data: Option<DataV2>,
    primary_sale_happened: Option<bool>,
    is_mutable: Option<bool>,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(metadata_account, false),
            AccountMeta::new_readonly(update_authority, true),
        ],
        data: MetadataInstruction::UpdateMetadataAccountV2(UpdateMetadataAccountArgsV2 {
            data,
            update_authority: new_update_authority,
            primary_sale_happened,
            is_mutable,
        })
        .try_to_vec()
        .unwrap(),
    }
}

/// creates a update_primary_sale_happened_via_token instruction
#[allow(clippy::too_many_arguments)]
pub fn update_primary_sale_happened_via_token(
    program_id: Pubkey,
    metadata: Pubkey,
    owner: Pubkey,
    token: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(metadata, false),
            AccountMeta::new_readonly(owner, true),
            AccountMeta::new_readonly(token, false),
        ],
        data: MetadataInstruction::UpdatePrimarySaleHappenedViaToken
            .try_to_vec()
            .unwrap(),
    }
}

/// Creates an instruction to mint a new asset and associated metadata accounts.
///
/// # Accounts:
///
///   0. `[writable]` Token account
///   1. `[writable]` Metadata account
///   2. `[]` Mint account
///   3. `[signer]` Mint authority
///   4. `[signer]` Payer
///   5. `[signer]` Update authority
///   6. `[]` System program
///   7. `[]` Instructions sysvar account
///   8. `[optional]` Asset authorization rules account
#[allow(clippy::too_many_arguments)]
pub fn mint(
    program_id: Pubkey,
    token_account: Pubkey,
    metadata_account: Pubkey,
    mint_account: Pubkey,
    mint_authority: Pubkey,
    payer: Pubkey,
    update_authority: Pubkey,
    data: AssetData,
    update_authority_as_signer: bool,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(token_account, false),
        AccountMeta::new(metadata_account, false),
        AccountMeta::new_readonly(mint_account, false),
        AccountMeta::new_readonly(mint_authority, true),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(update_authority, update_authority_as_signer),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(sysvar::id(), false),
    ];

    if let Some(config) = &data.programmable_config {
        accounts.push(AccountMeta::new_readonly(config.rule_set, false));
    }

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::Mint(MintArgs::V1(data))
            .try_to_vec()
            .unwrap(),
    }
}

/// Creates an instruction to mint a new asset and associated metadata accounts.
///
/// # Accounts:
///   0. `[writable]` Token account
///   1. `[writable]` Metadata account
///   2. `[]` Mint account
///   3. `[]` Destination account
///   4. `[writable]` Destination associate token account
///   5. `[signer]` Owner
///   6. `[]` STL Token program
///   7. `[]` STL Associate Token program
///   8. `[]` System program
///   9. `[]` Instructions sysvar account
///   10. `[optional]` Asset authorization rules account
///   11. `[optional]` Token Authorization Rules program
#[allow(clippy::too_many_arguments)]
pub fn transfer(
    program_id: Pubkey,
    token_account: Pubkey,
    metadata_account: Pubkey,
    mint_account: Pubkey,
    destination: Pubkey,
    destination_token_account: Pubkey,
    owner: Pubkey,
    args: TransferArgs,
    authorization_rules: Option<Pubkey>,
    additional_accounts: Option<Vec<AccountMeta>>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(token_account, false),
        AccountMeta::new(metadata_account, false),
        AccountMeta::new_readonly(mint_account, false),
        AccountMeta::new_readonly(destination, false),
        AccountMeta::new(destination_token_account, false),
        AccountMeta::new(owner, true),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        AccountMeta::new_readonly(sysvar::id(), false),
    ];

    if let Some(authorization_rules) = authorization_rules {
        accounts.push(AccountMeta::new_readonly(authorization_rules, false));
        //accounts.push(AccountMeta::new_readonly(token_authorization::id(), false));
    }

    if let Some(additional_accounts) = additional_accounts {
        accounts.extend(additional_accounts);
    }

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::Transfer(args).try_to_vec().unwrap(),
    }
}
