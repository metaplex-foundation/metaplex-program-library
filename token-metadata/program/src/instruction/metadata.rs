use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};
#[cfg(feature = "serde-feature")]
use {
    serde::{Deserialize, Serialize},
    serde_with::{As, DisplayFromStr},
};

use super::InstructionBuilder;
use crate::{
    instruction::MetadataInstruction,
    processor::AuthorizationData,
    state::{
        AssetData, Collection, CollectionDetails, Creator, Data, DataV2, DelegateState,
        ProgrammableConfig, TokenStandard, Uses,
    },
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
pub enum CreateArgs {
    V1 {
        asset_data: AssetData,
        decimals: Option<u8>,
        max_supply: Option<u64>,
    },
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum MintArgs {
    V1 {
        amount: u64,
        authorization_data: Option<AuthorizationData>,
    },
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum TransferArgs {
    V1 {
        authorization_data: Option<AuthorizationData>,
        amount: u64,
    },
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum UpdateArgs {
    V1 {
        authorization_data: Option<AuthorizationData>,
        new_update_authority: Option<Pubkey>,
        data: Option<Data>,
        primary_sale_happened: Option<bool>,
        is_mutable: Option<bool>,
        token_standard: Option<TokenStandard>,
        collection: Option<Collection>,
        uses: Option<Uses>,
        collection_details: Option<CollectionDetails>,
        programmable_config: Option<ProgrammableConfig>,
        delegate_state: Option<DelegateState>,
        authority_type: AuthorityType,
    },
}

impl Default for UpdateArgs {
    fn default() -> Self {
        Self::V1 {
            authorization_data: None,
            new_update_authority: None,
            data: None,
            primary_sale_happened: None,
            is_mutable: None,
            token_standard: None,
            collection: None,
            uses: None,
            collection_details: None,
            programmable_config: None,
            delegate_state: None,
            authority_type: AuthorityType::Metadata,
        }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum MigrateArgs {
    V1,
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

/// Builds the instruction to create metadata and associated accounts.
///
/// # Accounts:
///
///   0. `[writable]` Metadata account
///   1. `[]` Mint account
///   2. `[signer]` Mint authority
///   3. `[signer]` Payer
///   4. `[signer]` Update authority
///   5. `[]` System program
///   6. `[]` Instructions sysvar account
///   7. `[]` SPL Token program
///   8. `[optional]` Master edition account
///   9. `[optional]` Asset authorization rules account
impl InstructionBuilder for super::builders::Create {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let mut accounts = vec![
            AccountMeta::new(self.metadata, false),
            if self.initialize_mint {
                AccountMeta::new(self.mint, true)
            } else {
                // even with an existing mint, we require the account to be writable since
                // in some cases the mint authority will be updated
                AccountMeta::new(self.mint, false)
            },
            AccountMeta::new_readonly(self.mint_authority, true),
            AccountMeta::new(self.payer, true),
            AccountMeta::new_readonly(self.update_authority, self.update_authority_as_signer),
            AccountMeta::new_readonly(self.system_program, false),
            AccountMeta::new_readonly(self.sysvar_instructions, false),
            AccountMeta::new_readonly(self.spl_token_program, false),
        ];
        // checks whether we have a master edition
        if let Some(master_edition) = self.master_edition {
            accounts.push(AccountMeta::new(master_edition, false));
        } else {
            accounts.push(AccountMeta::new_readonly(crate::ID, false));
        }
        // checks whether we have authorization rules
        if let Some(rules) = &self.authorization_rules {
            accounts.push(AccountMeta::new_readonly(*rules, false));
        } else {
            accounts.push(AccountMeta::new_readonly(crate::ID, false));
        }

        Instruction {
            program_id: crate::ID,
            accounts,
            data: MetadataInstruction::Create(self.args.clone())
                .try_to_vec()
                .unwrap(),
        }
    }
}

/// Mints tokens from a mint account.
///
/// # Accounts:
///
///   0. `[writable`] Token account key
///   1. `[]` Metadata account key (pda of ['metadata', program id, mint id])")]
///   2. `[writable]` Mint of token asset
///   3. `[signer, writable]` Payer
///   4. `[signer]` Authority (mint authority or metadata's update authority for NonFungible asests)
///   5. `[]` System program
///   6. `[]` Instructions sysvar account
///   7. `[]` SPL Token program
///   8. `[]` SPL Associated Token Account program
///   9. `[optional]` Master Edition account
///   10. `[optional]` Token Authorization Rules program
///   11. `[optional]` Token Authorization Rules account
pub fn mint(
    token: Pubkey,
    metadata: Pubkey,
    mint: Pubkey,
    payer: Pubkey,
    authority: Pubkey,
    master_edition: Option<Pubkey>,
    authorization_rules: Option<Pubkey>,
    args: MintArgs,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(token, false),
        AccountMeta::new_readonly(metadata, false),
        AccountMeta::new(mint, false),
        AccountMeta::new(payer, true),
        AccountMeta::new(authority, true),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(sysvar::instructions::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
    ];
    // checks whether we have a master edition
    if let Some(master_edition) = master_edition {
        accounts.push(AccountMeta::new(master_edition, false));
    } else {
        accounts.push(AccountMeta::new_readonly(crate::id(), false));
    }
    // checks whether we have authorization rules
    if let Some(authorization_rules) = authorization_rules {
        accounts.push(AccountMeta::new(authorization_rules, false));
        accounts.push(AccountMeta::new_readonly(mpl_token_auth_rules::id(), false));
    } else {
        accounts.push(AccountMeta::new_readonly(crate::id(), false));
        accounts.push(AccountMeta::new_readonly(crate::id(), false));
    }

    Instruction {
        program_id: crate::id(),
        accounts,
        data: MetadataInstruction::Mint(args).try_to_vec().unwrap(),
    }
}

/// Creates an instruction to mint a new asset and associated metadata accounts.
///
/// # Accounts:
///   0. `[writable]` Token account
///   1. `[writable]` Metadata account
///   2. `[]` Mint account
///   5. `[signer]` Owner
///   4. `[writable]` Destination associate token account
///   3. `[]` Destination owner
///   6. `[]` SPL Token program
///   7. `[]` SPL Associate Token program
///   8. `[]` System programe
///   9. `[]` Instructions sysvar account
///   10. `[optional]` Asset authorization rules account
///   11. `[optional]` Token Authorization Rules program
#[allow(clippy::too_many_arguments)]
pub fn transfer(
    program_id: Pubkey,
    owner: Pubkey,
    token: Pubkey,
    metadata_account: Pubkey,
    mint_account: Pubkey,
    edition: Option<Pubkey>,
    destination_owner: Pubkey,
    destination_token: Pubkey,
    args: TransferArgs,
    authorization_rules: Option<Pubkey>,
    additional_accounts: Option<Vec<AccountMeta>>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(owner, true),
        AccountMeta::new(token, false),
        AccountMeta::new(metadata_account, false),
        AccountMeta::new_readonly(mint_account, false),
        AccountMeta::new(edition.unwrap_or(crate::ID), false),
        AccountMeta::new(destination_owner, false),
        AccountMeta::new(destination_token, false),
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        AccountMeta::new_readonly(solana_program::system_program::ID, false),
        AccountMeta::new_readonly(sysvar::instructions::ID, false),
        AccountMeta::new_readonly(mpl_token_auth_rules::ID, false),
        AccountMeta::new_readonly(authorization_rules.unwrap_or(crate::ID), false),
    ];

    if let Some(additional_accounts) = additional_accounts {
        accounts.extend(additional_accounts);
    }

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::Transfer(args).try_to_vec().unwrap(),
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum AuthorityType {
    Metadata,
    Delegate,
    Holder,
    Other,
}

/// Creates an instruction to update an existing asset.
///
/// # Accounts:
///
///   0. `[writable]` Metadata account
///   1. `[]` Mint account
///   2. `[]` System program
///   3. `[]` Instructions sysvar account
///   4. `[optional]` Master edition account
///   5. `[optional]` New update authority
///   6. `[signer, optional]` Update authority
///   7. `[signer, optional]` Token holder
///   8. `[optional]` Token account
///   9. `[optional]` Asset authorization rules account
///   10. `[optional]` Authorization rules program
#[allow(clippy::too_many_arguments)]
pub fn update(
    program_id: Pubkey,
    metadata_account: Pubkey,
    mint_account: Pubkey,
    master_edition_account: Option<Pubkey>,
    authority: Pubkey,
    token_account: Option<Pubkey>,
    delegate_record: Option<Pubkey>,
    authorization_rules: Option<Pubkey>,
    additional_accounts: Option<Vec<AccountMeta>>,
    args: UpdateArgs,
) -> Instruction {
    let mut accounts: Vec<AccountMeta> = vec![
        AccountMeta::new(metadata_account, false),
        AccountMeta::new_readonly(mint_account, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(sysvar::instructions::id(), false),
        AccountMeta::new(master_edition_account.unwrap_or(crate::ID), false),
        AccountMeta::new_readonly(authority, true),
        AccountMeta::new_readonly(token_account.unwrap_or(crate::ID), false),
        AccountMeta::new_readonly(delegate_record.unwrap_or(crate::ID), false),
    ];

    if let Some(authorization_rules) = authorization_rules {
        accounts.push(AccountMeta::new_readonly(mpl_token_auth_rules::ID, false));
        accounts.push(AccountMeta::new_readonly(authorization_rules, false));
    } else {
        accounts.push(AccountMeta::new_readonly(crate::ID, false));
        accounts.push(AccountMeta::new_readonly(crate::ID, false));
    }

    if let Some(additional_accounts) = additional_accounts {
        accounts.extend(additional_accounts);
    }

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::Update(args).try_to_vec().unwrap(),
    }
}

/// Creates an instruction to migrate an asset to a ProgrammableAsset.
///
/// # Accounts:
///
///   0. `[writable]` Metadata account
///   1. `[]` Master edition account
///   2. `[writable]` Token account
///   3. `[]` Mint account
///   4. `[signer]` Update authority
///   6. `[]` Collection metadata account
///   8. `[]` Token Program
///   7. `[]` System program
///   9. `[]` Instruction sysvar account
//   10. optional, name="authorization_rules", desc="Token Authorization Rules account"
#[allow(clippy::too_many_arguments)]
pub fn migrate(
    program_id: Pubkey,
    metadata_account: Pubkey,
    master_edition_account: Pubkey,
    mint: Pubkey,
    token_account: Pubkey,
    update_authority: Pubkey,
    collection_metadata: Pubkey,
    authorization_rules: Option<Pubkey>,
    additional_accounts: Option<Vec<AccountMeta>>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(metadata_account, false),
        AccountMeta::new_readonly(master_edition_account, false),
        AccountMeta::new(token_account, false),
        AccountMeta::new_readonly(mint, false),
        AccountMeta::new_readonly(update_authority, true),
        AccountMeta::new_readonly(collection_metadata, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::instructions::id(), false),
    ];

    if let Some(authorization_rules) = authorization_rules {
        accounts.push(AccountMeta::new_readonly(authorization_rules, false));
        //accounts.push(AccountMeta::new_readonly(token_authorization::id(), false));
    }

    accounts.extend(additional_accounts.unwrap_or_default());

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::Migrate(MigrateArgs::V1)
            .try_to_vec()
            .unwrap(),
    }
}
