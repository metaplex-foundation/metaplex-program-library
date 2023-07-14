use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
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
        AssetData, Collection, CollectionDetails, Creator, Data, DataV2, PrintSupply,
        TokenStandard, Uses,
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
        print_supply: Option<PrintSupply>,
    },
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum MintArgs {
    V1 {
        amount: u64,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum TransferArgs {
    V1 {
        amount: u64,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
}

/// Struct representing the values to be updated for an `update` instructions.
///
/// Values that are set to `None` are not changed.  Any value set to `Some(...)` will
/// have its value updated. There are properties that have three valid states, and
/// use a "toggle" type that allows the value to be set, cleared, or remain the same.
#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum UpdateArgs {
    V1 {
        /// The new update authority.
        new_update_authority: Option<Pubkey>,
        /// The metadata details.
        data: Option<Data>,
        /// Indicates whether the primary sale has happened or not (once set to `true`, it cannot be
        /// changed back).
        primary_sale_happened: Option<bool>,
        // Indicates Whether the data struct is mutable or not (once set to `true`, it cannot be
        /// changed back).
        is_mutable: Option<bool>,
        /// Collection information.
        collection: CollectionToggle,
        /// Additional details of the collection.
        collection_details: CollectionDetailsToggle,
        /// Uses information.
        uses: UsesToggle,
        // Programmable rule set configuration (only applicable to `Programmable` asset types).
        rule_set: RuleSetToggle,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    AsUpdateAuthorityV2 {
        /// The new update authority.
        new_update_authority: Option<Pubkey>,
        /// The metadata details.
        data: Option<Data>,
        /// Indicates whether the primary sale has happened or not (once set to `true`, it cannot be
        /// changed back).
        primary_sale_happened: Option<bool>,
        // Indicates Whether the data struct is mutable or not (once set to `true`, it cannot be
        /// changed back).
        is_mutable: Option<bool>,
        /// Collection information.
        collection: CollectionToggle,
        /// Additional details of the collection.
        collection_details: CollectionDetailsToggle,
        /// Uses information.
        uses: UsesToggle,
        // Programmable rule set configuration (only applicable to `Programmable` asset types).
        rule_set: RuleSetToggle,
        /// Token standard.
        token_standard: Option<TokenStandard>,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    AsAuthorityItemDelegateV2 {
        /// The new update authority.
        new_update_authority: Option<Pubkey>,
        /// Indicates whether the primary sale has happened or not (once set to `true`, it cannot be
        /// changed back).
        primary_sale_happened: Option<bool>,
        // Indicates Whether the data struct is mutable or not (once set to `true`, it cannot be
        /// changed back).
        is_mutable: Option<bool>,
        /// Token standard.
        token_standard: Option<TokenStandard>,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    AsCollectionDelegateV2 {
        /// Collection information.
        collection: CollectionToggle,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    AsDataDelegateV2 {
        /// The metadata details.
        data: Option<Data>,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    AsProgrammableConfigDelegateV2 {
        // Programmable rule set configuration (only applicable to `Programmable` asset types).
        rule_set: RuleSetToggle,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    AsDataItemDelegateV2 {
        /// The metadata details.
        data: Option<Data>,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    AsCollectionItemDelegateV2 {
        /// Collection information.
        collection: CollectionToggle,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    AsProgrammableConfigItemDelegateV2 {
        // Programmable rule set configuration (only applicable to `Programmable` asset types).
        rule_set: RuleSetToggle,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
}

impl UpdateArgs {
    pub fn default_v1() -> Self {
        Self::V1 {
            new_update_authority: None,
            data: None,
            primary_sale_happened: None,
            is_mutable: None,
            collection: CollectionToggle::default(),
            collection_details: CollectionDetailsToggle::default(),
            uses: UsesToggle::default(),
            rule_set: RuleSetToggle::default(),
            authorization_data: None,
        }
    }

    pub fn default_as_update_authority() -> Self {
        Self::AsUpdateAuthorityV2 {
            new_update_authority: None,
            data: None,
            primary_sale_happened: None,
            is_mutable: None,
            collection: CollectionToggle::default(),
            collection_details: CollectionDetailsToggle::default(),
            uses: UsesToggle::default(),
            rule_set: RuleSetToggle::default(),
            token_standard: None,
            authorization_data: None,
        }
    }

    pub fn default_as_authority_item_delegate() -> Self {
        Self::AsAuthorityItemDelegateV2 {
            new_update_authority: None,
            primary_sale_happened: None,
            is_mutable: None,
            token_standard: None,
            authorization_data: None,
        }
    }

    pub fn default_as_collection_delegate() -> Self {
        Self::AsCollectionDelegateV2 {
            collection: CollectionToggle::default(),
            authorization_data: None,
        }
    }

    pub fn default_as_data_delegate() -> Self {
        Self::AsDataDelegateV2 {
            data: None,
            authorization_data: None,
        }
    }

    pub fn default_as_programmable_config_delegate() -> Self {
        Self::AsProgrammableConfigDelegateV2 {
            rule_set: RuleSetToggle::default(),
            authorization_data: None,
        }
    }

    pub fn default_as_data_item_delegate() -> Self {
        Self::AsDataItemDelegateV2 {
            data: None,
            authorization_data: None,
        }
    }

    pub fn default_as_collection_item_delegate() -> Self {
        Self::AsCollectionItemDelegateV2 {
            collection: CollectionToggle::default(),
            authorization_data: None,
        }
    }

    pub fn default_as_programmable_config_item_delegate() -> Self {
        Self::AsProgrammableConfigItemDelegateV2 {
            rule_set: RuleSetToggle::default(),
            authorization_data: None,
        }
    }
}

//-- Toggle implementations

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
pub enum CollectionToggle {
    #[default]
    None,
    Clear,
    Set(Collection),
}

impl CollectionToggle {
    pub fn is_some(&self) -> bool {
        matches!(self, CollectionToggle::Clear | CollectionToggle::Set(_))
    }

    pub fn is_none(&self) -> bool {
        matches!(self, CollectionToggle::None)
    }

    pub fn is_clear(&self) -> bool {
        matches!(self, CollectionToggle::Clear)
    }

    pub fn is_set(&self) -> bool {
        matches!(self, CollectionToggle::Set(_))
    }

    pub fn to_option(self) -> Option<Collection> {
        match self {
            CollectionToggle::Set(value) => Some(value),
            CollectionToggle::Clear => None,
            CollectionToggle::None => panic!("Tried to convert 'None' value"),
        }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
pub enum UsesToggle {
    #[default]
    None,
    Clear,
    Set(Uses),
}

impl UsesToggle {
    pub fn is_some(&self) -> bool {
        matches!(self, UsesToggle::Clear | UsesToggle::Set(_))
    }

    pub fn is_none(&self) -> bool {
        matches!(self, UsesToggle::None)
    }

    pub fn is_clear(&self) -> bool {
        matches!(self, UsesToggle::Clear)
    }

    pub fn is_set(&self) -> bool {
        matches!(self, UsesToggle::Set(_))
    }

    pub fn to_option(self) -> Option<Uses> {
        match self {
            UsesToggle::Set(value) => Some(value),
            UsesToggle::Clear => None,
            UsesToggle::None => panic!("Tried to convert 'None' value"),
        }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
pub enum CollectionDetailsToggle {
    #[default]
    None,
    Clear,
    Set(CollectionDetails),
}

impl CollectionDetailsToggle {
    pub fn is_some(&self) -> bool {
        matches!(
            self,
            CollectionDetailsToggle::Clear | CollectionDetailsToggle::Set(_)
        )
    }

    pub fn is_none(&self) -> bool {
        matches!(self, CollectionDetailsToggle::None)
    }

    pub fn is_clear(&self) -> bool {
        matches!(self, CollectionDetailsToggle::Clear)
    }

    pub fn is_set(&self) -> bool {
        matches!(self, CollectionDetailsToggle::Set(_))
    }

    pub fn to_option(self) -> Option<CollectionDetails> {
        match self {
            CollectionDetailsToggle::Set(value) => Some(value),
            CollectionDetailsToggle::Clear => None,
            CollectionDetailsToggle::None => panic!("Tried to convert 'None' value"),
        }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
pub enum RuleSetToggle {
    #[default]
    None,
    Clear,
    Set(Pubkey),
}

impl RuleSetToggle {
    pub fn is_some(&self) -> bool {
        matches!(self, RuleSetToggle::Clear | RuleSetToggle::Set(_))
    }

    pub fn is_none(&self) -> bool {
        matches!(self, RuleSetToggle::None)
    }

    pub fn is_clear(&self) -> bool {
        matches!(self, RuleSetToggle::Clear)
    }

    pub fn is_set(&self) -> bool {
        matches!(self, RuleSetToggle::Set(_))
    }

    pub fn to_option(self) -> Option<Pubkey> {
        match self {
            RuleSetToggle::Set(t) => Some(t),
            RuleSetToggle::Clear => None,
            RuleSetToggle::None => panic!("Tried to convert 'None' value"),
        }
    }
}

//-- End Toggle implementation

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

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum PrintArgs {
    V1 { edition: u64 },
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
            AccountMeta::new_readonly(system_program::ID, false),
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
        AccountMeta::new_readonly(update_authority, true),
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

//-- Instruction Builders trait implementation

/// Builds the instruction to create metadata and associated accounts.
///
/// # Accounts:
///
///   0. `[writable]` Metadata account
///   1. `[optional, writable]` Master edition account
///   2. `[writable]` Mint account
///   3. `[signer]` Mint authority
///   4. `[signer]` Payer
///   5. `[signer]` Update authority
///   6. `[]` System program
///   7. `[]` Instructions sysvar account
///   8. `[]` SPL Token program
impl InstructionBuilder for super::builders::Create {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let accounts = vec![
            AccountMeta::new(self.metadata, false),
            // checks whether we have a master edition
            if let Some(master_edition) = self.master_edition {
                AccountMeta::new(master_edition, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            AccountMeta::new(self.mint, self.initialize_mint),
            AccountMeta::new_readonly(self.authority, true),
            AccountMeta::new(self.payer, true),
            AccountMeta::new_readonly(self.update_authority, self.update_authority_as_signer),
            AccountMeta::new_readonly(self.system_program, false),
            AccountMeta::new_readonly(self.sysvar_instructions, false),
            AccountMeta::new_readonly(self.spl_token_program, false),
        ];

        Instruction {
            program_id: crate::ID,
            accounts,
            data: MetadataInstruction::Create(self.args.clone())
                .try_to_vec()
                .unwrap(),
        }
    }
}

/// Builds the instruction to mint a token.
///
/// # Accounts:
///
///   0. `[writable]` Token account key
///   1. `[optional]` Owner of the token account
///   2. `[]` Metadata account key (pda of ['metadata', program id, mint id])")]
///   3. `[optional]` Master Edition account
///   4. `[optional]` Token record account
///   5. `[writable]` Mint of token asset
///   6. `[signer]` Authority (mint authority or metadata's update authority for NonFungible asests)
///   7. `[optional]` Metadata delegate record
///   8. `[signer, writable]` Payer
///   9. `[]` System program
///   10. `[]` Instructions sysvar account
///   11. `[]` SPL Token program
///   12. `[]` SPL Associated Token Account program
///   13. `[optional]` Token Authorization Rules program
///   14. `[optional]` Token Authorization Rules account
impl InstructionBuilder for super::builders::Mint {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let mut accounts = vec![
            AccountMeta::new(self.token, false),
            AccountMeta::new_readonly(self.token_owner.unwrap_or(crate::ID), false),
            AccountMeta::new_readonly(self.metadata, false),
            if let Some(master_edition) = self.master_edition {
                AccountMeta::new(master_edition, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            if let Some(token_record) = self.token_record {
                AccountMeta::new(token_record, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            AccountMeta::new(self.mint, false),
            AccountMeta::new_readonly(self.authority, true),
            AccountMeta::new_readonly(self.delegate_record.unwrap_or(crate::ID), false),
            AccountMeta::new(self.payer, true),
            AccountMeta::new_readonly(self.system_program, false),
            AccountMeta::new_readonly(self.sysvar_instructions, false),
            AccountMeta::new_readonly(self.spl_token_program, false),
            AccountMeta::new_readonly(self.spl_ata_program, false),
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
            data: MetadataInstruction::Mint(self.args.clone())
                .try_to_vec()
                .unwrap(),
        }
    }
}

/// Transfer tokens from a token account.
///
/// # Accounts:
///
///   0. `[writable]` Token account
///   1. `[]` Token account owner
///   2. `[writable]` Destination token account
///   3. `[]` Destination token account owner
///   4. `[]` Mint of token asset
///   5. `[writable]` Metadata account
///   6. `[optional]` Edition of token asset
///   7. `[optional, writable]` Owner token record account
///   8. `[optional, writable]` Destination token record account
///   9. `[signer]` Transfer authority (token owner or delegate)
///   10. `[signer, writable]` Payer
///   11. `[]` System Program
///   12. `[]` Instructions sysvar account
///   13. `[]` SPL Token Program
///   14. `[]` SPL Associated Token Account program
///   15. `[optional]` Token Authorization Rules Program
///   16. `[optional]` Token Authorization Rules account
impl InstructionBuilder for super::builders::Transfer {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let mut accounts = vec![
            AccountMeta::new(self.token, false),
            AccountMeta::new_readonly(self.token_owner, false),
            AccountMeta::new(self.destination, false),
            AccountMeta::new_readonly(self.destination_owner, false),
            AccountMeta::new_readonly(self.mint, false),
            AccountMeta::new(self.metadata, false),
            AccountMeta::new_readonly(self.edition.unwrap_or(crate::ID), false),
            if let Some(owner_token_record) = self.owner_token_record {
                AccountMeta::new(owner_token_record, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            if let Some(destination_token_record) = self.destination_token_record {
                AccountMeta::new(destination_token_record, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            AccountMeta::new_readonly(self.authority, true),
            AccountMeta::new(self.payer, true),
            AccountMeta::new_readonly(self.system_program, false),
            AccountMeta::new_readonly(self.sysvar_instructions, false),
            AccountMeta::new_readonly(self.spl_token_program, false),
            AccountMeta::new_readonly(self.spl_ata_program, false),
        ];
        // Optional authorization rules accounts
        if let Some(rules) = &self.authorization_rules {
            accounts.push(AccountMeta::new_readonly(
                self.authorization_rules_program.unwrap_or(crate::ID),
                false,
            ));
            accounts.push(AccountMeta::new_readonly(*rules, false));
        } else {
            accounts.push(AccountMeta::new_readonly(crate::ID, false));
            accounts.push(AccountMeta::new_readonly(crate::ID, false));
        }

        Instruction {
            program_id: crate::ID,
            accounts,
            data: MetadataInstruction::Transfer(self.args.clone())
                .try_to_vec()
                .unwrap(),
        }
    }
}

/// Updates the metadata of an asset.
///
/// # Accounts:
///
///   0. `[signer]` Update authority or delegate
///   1. `[optional]` Delegate record PDA
///   2. `[optional]` Token account
///   3. `[]` Mint account
///   4. `[writable]` Metadata account
///   5. `[optional]` Edition account
///   6. `[signer]` Payer
///   7. `[]` System program
///   8. `[]` System program
///   9. `[optional]` Token Authorization Rules Program
///   10. `[optional]` Token Authorization Rules account
impl InstructionBuilder for super::builders::Update {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let mut accounts = vec![
            AccountMeta::new_readonly(self.authority, true),
            AccountMeta::new_readonly(self.delegate_record.unwrap_or(crate::ID), false),
            AccountMeta::new_readonly(self.token.unwrap_or(crate::ID), false),
            AccountMeta::new_readonly(self.mint, false),
            AccountMeta::new(self.metadata, false),
            AccountMeta::new_readonly(self.edition.unwrap_or(crate::ID), false),
            AccountMeta::new(self.payer, true),
            AccountMeta::new_readonly(self.system_program, false),
            AccountMeta::new_readonly(self.sysvar_instructions, false),
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
            data: MetadataInstruction::Update(self.args.clone())
                .try_to_vec()
                .unwrap(),
        }
    }
}

/// Prints an edition from a master edition.
///
/// # Accounts:
///
///   0. `[writable]` Edition Metadata
///   1. `[writable]` Edition
///   2. `[writable, signer]` Edition Mint
///   3. `[]` Edition Token Account Owner
///   4. `[writable]` Edition Token Account
///   5. `[signer]` Edition Mint Authority
///   6. `[writable]` Edition Token Record
///   7. `[writable]`, Master Edition
///   8. `[writable]`, Edition Marker
///   9. `[signer, writable]` Payer
///   10. `[signer]`, Master Token Account Owner
///   11. `[]` Master Token Account
///   12. `[]` Master Metadata
///   12. `[]` Update Authority
///   13. `[]` Token Program
///   14. `[]` Associated Token Account Program
///   15. `[]` Instructions System Variable
///   16. `[]` System Program

impl InstructionBuilder for super::builders::Print {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let accounts = vec![
            AccountMeta::new(self.edition_metadata, false),
            AccountMeta::new(self.edition, false),
            AccountMeta::new(self.edition_mint, self.initialize_mint),
            AccountMeta::new_readonly(self.edition_token_account_owner, false),
            AccountMeta::new(self.edition_token_account, false),
            AccountMeta::new_readonly(self.edition_mint_authority, true),
            if let Some(edition_token_record) = self.edition_token_record {
                AccountMeta::new(edition_token_record, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            AccountMeta::new(self.master_edition, false),
            AccountMeta::new(self.edition_marker_pda, false),
            AccountMeta::new(self.payer, true),
            AccountMeta::new_readonly(self.master_token_account_owner, true),
            AccountMeta::new_readonly(self.master_token_account, false),
            AccountMeta::new_readonly(self.master_metadata, false),
            AccountMeta::new_readonly(self.update_authority, false),
            AccountMeta::new_readonly(self.spl_token_program, false),
            AccountMeta::new_readonly(self.spl_ata_program, false),
            AccountMeta::new_readonly(self.sysvar_instructions, false),
            AccountMeta::new_readonly(self.system_program, false),
        ];

        Instruction {
            program_id: crate::ID,
            accounts,
            data: MetadataInstruction::Print(self.args.clone())
                .try_to_vec()
                .unwrap(),
        }
    }
}
