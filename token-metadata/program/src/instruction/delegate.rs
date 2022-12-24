use std::fmt;

use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};
use solana_program::instruction::{AccountMeta, Instruction};

use super::InstructionBuilder;
use crate::instruction::MetadataInstruction;

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum DelegateArgs {
    CollectionV1,
    SaleV1 { amount: u64 },
    TransferV1 { amount: u64 },
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum RevokeArgs {
    CollectionV1,
    TransferV1,
    SaleV1,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum DelegateRole {
    Authority,
    Collection,
    Transfer,
    Use,
    Utility,
    Sale,
    Update,
}

impl fmt::Display for DelegateRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            Self::Authority => "authority_delegate".to_string(),
            Self::Collection => "collection_delegate".to_string(),
            Self::Transfer => "sale_delegate".to_string(),
            Self::Use => "use_delegate".to_string(),
            Self::Utility => "utility_delegate".to_string(),
            Self::Sale => "sale_delegate".to_string(),
            Self::Update => "update_delegate".to_string(),
        };

        write!(f, "{}", message)
    }
}

/// Delegates an action over an asset to a specific account.
///
/// # Accounts:
///
///   0. `[writable]` Delegate account key
///   1. `[]` Delegated owner
///   2. `[writable]` Metadata account
///   3. `[optional]` Master Edition account
///   4. `[]` Mint account
///   5. `[optional, writable]` Token account
///   6. `[signer]` Authority to approve the delegation
///   7. `[signer, writable]` Payer
///   8. `[]` System Program
///   9. `[]` Instructions sysvar account
///   10. `[optional]` SPL Token Program
///   11. `[optional]` Token Authorization Rules program
///   12. `[optional]` Token Authorization Rules account
impl InstructionBuilder for super::builders::Delegate {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let accounts = vec![
            AccountMeta::new(self.delegate_record, false),
            AccountMeta::new_readonly(self.delegate, false),
            AccountMeta::new(self.metadata, false),
            AccountMeta::new_readonly(self.master_edition.unwrap_or(crate::ID), false),
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
///   0. `[writable]` Delegate account key
///   1. `[]` Delegated owner
///   2. `[writable]` Metadata account
///   3. `[optional]` Master Edition account
///   4. `[]` Mint account
///   5. `[optional, writable]` Token account
///   6. `[signer]` Authority to approve the delegation
///   7. `[signer, writable]` Payer
///   8. `[]` System Program
///   9. `[]` Instructions sysvar account
///   10. `[optional]` SPL Token Program
///   11. `[optional]` Token Authorization Rules program
///   12. `[optional]` Token Authorization Rules account
impl InstructionBuilder for super::builders::Revoke {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let accounts = vec![
            AccountMeta::new(self.delegate_record, false),
            AccountMeta::new_readonly(self.delegate, false),
            AccountMeta::new(self.metadata, false),
            AccountMeta::new_readonly(self.master_edition.unwrap_or(crate::ID), false),
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
