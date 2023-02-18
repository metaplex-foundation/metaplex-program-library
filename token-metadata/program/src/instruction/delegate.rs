use std::fmt;

use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use super::InstructionBuilder;
use crate::{instruction::MetadataInstruction, processor::AuthorizationData};

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum DelegateArgs {
    CollectionV1 {
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    SaleV1 {
        amount: u64,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    TransferV1 {
        amount: u64,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    UpdateV1 {
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    UtilityV1 {
        amount: u64,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    StakingV1 {
        amount: u64,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    StandardV1 {
        amount: u64,
    },
    LockedTransferV1 {
        amount: u64,
        /// locked destination pubkey
        locked_address: Pubkey,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    ProgrammableConfigV1 {
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum RevokeArgs {
    CollectionV1,
    SaleV1,
    TransferV1,
    UpdateV1,
    UtilityV1,
    StakingV1,
    StandardV1,
    LockedTransferV1,
    ProgrammableConfigV1,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum MetadataDelegateRole {
    Authority,
    Collection,
    Use,
    Update,
    ProgrammableConfig,
}

impl fmt::Display for MetadataDelegateRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            Self::Authority => "authority_delegate".to_string(),
            Self::Collection => "collection_delegate".to_string(),
            Self::Use => "use_delegate".to_string(),
            Self::Update => "update_delegate".to_string(),
            Self::ProgrammableConfig => "programmable_config_delegate".to_string(),
        };

        write!(f, "{message}")
    }
}

/// Delegates an action over an asset to a specific account.
///
/// # Accounts:
///
///   0. `[optional, writable]` Delegate record account
///   1. `[]` Delegated owner
///   2. `[writable]` Metadata account
///   3. `[optional]` Master Edition account
///   4. `[optional, writable]` Token record account
///   5. `[]` Mint account
///   6. `[optional, writable]` Token account
///   7. `[signer]` Update authority or token owner
///   8. `[signer, writable]` Payer
///   9. `[]` System Program
///   10. `[]` Instructions sysvar account
///   11. `[optional]` SPL Token Program
///   12. `[optional]` Token Authorization Rules program
///   13. `[optional]` Token Authorization Rules account
impl InstructionBuilder for super::builders::Delegate {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let accounts = vec![
            if let Some(delegate_record) = self.delegate_record {
                AccountMeta::new(delegate_record, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            AccountMeta::new_readonly(self.delegate, false),
            AccountMeta::new(self.metadata, false),
            AccountMeta::new_readonly(self.master_edition.unwrap_or(crate::ID), false),
            if let Some(token_record) = self.token_record {
                AccountMeta::new(token_record, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            AccountMeta::new_readonly(self.mint, false),
            if let Some(token) = self.token {
                AccountMeta::new(token, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            AccountMeta::new_readonly(self.authority, true),
            AccountMeta::new(self.payer, true),
            AccountMeta::new_readonly(self.system_program, false),
            AccountMeta::new_readonly(self.sysvar_instructions, false),
            AccountMeta::new_readonly(self.spl_token_program.unwrap_or(crate::ID), false),
            AccountMeta::new_readonly(self.authorization_rules_program.unwrap_or(crate::ID), false),
            AccountMeta::new_readonly(self.authorization_rules.unwrap_or(crate::ID), false),
        ];

        Instruction {
            program_id: crate::ID,
            accounts,
            data: MetadataInstruction::Delegate(self.args.clone())
                .try_to_vec()
                .unwrap(),
        }
    }
}

/// Revokes a delegate.
///
/// # Accounts:
///
///   0. `[optional, writable]` Delegate record account
///   1. `[]` Delegated owner
///   2. `[writable]` Metadata account
///   3. `[optional]` Master Edition account
///   4. `[optional, writable]` Token record account
///   5. `[]` Mint account
///   6. `[optional, writable]` Token account
///   7. `[signer]` Update authority or token owner
///   8. `[signer, writable]` Payer
///   9. `[]` System Program
///   10. `[]` Instructions sysvar account
///   11. `[optional]` SPL Token Program
///   12. `[optional]` Token Authorization Rules program
///   13. `[optional]` Token Authorization Rules account
impl InstructionBuilder for super::builders::Revoke {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let accounts = vec![
            if let Some(delegate_record) = self.delegate_record {
                AccountMeta::new(delegate_record, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            AccountMeta::new_readonly(self.delegate, false),
            AccountMeta::new(self.metadata, false),
            AccountMeta::new_readonly(self.master_edition.unwrap_or(crate::ID), false),
            if let Some(token_record) = self.token_record {
                AccountMeta::new(token_record, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            AccountMeta::new_readonly(self.mint, false),
            if let Some(token) = self.token {
                AccountMeta::new(token, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            AccountMeta::new_readonly(self.authority, true),
            AccountMeta::new(self.payer, true),
            AccountMeta::new_readonly(self.system_program, false),
            AccountMeta::new_readonly(self.sysvar_instructions, false),
            AccountMeta::new_readonly(self.spl_token_program.unwrap_or(crate::ID), false),
            AccountMeta::new_readonly(self.authorization_rules_program.unwrap_or(crate::ID), false),
            AccountMeta::new_readonly(self.authorization_rules.unwrap_or(crate::ID), false),
        ];

        Instruction {
            program_id: crate::ID,
            accounts,
            data: MetadataInstruction::Revoke(self.args.clone())
                .try_to_vec()
                .unwrap(),
        }
    }
}
