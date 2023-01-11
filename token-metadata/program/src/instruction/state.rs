use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};
use solana_program::instruction::{AccountMeta, Instruction};

use crate::processor::AuthorizationData;

use super::{InstructionBuilder, MetadataInstruction};

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum LockArgs {
    V1 {
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum UnlockArgs {
    V1 {
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
}

/// Locks an asset. For non-programmable assets, this will also freeze the token account.
///
/// # Accounts:
///
///   0. `[signer]` Token owner or delegate
///   1. `[writable, optional]` Delegate record account
///   2. `[writable, optional]` Token account
///   3. `[]` Mint account
///   4. `[writable]` Metadata account
///   5. `[optional]` Edition account
///   6. `[signer, writable]` Payer
///   7. `[]` System Program
///   8. `[]` Instructions sysvar account
///   9. `[optional]` SPL Token Program
///   10. `[optional]` Token Authorization Rules program
///   11. `[optional]` Token Authorization Rules account
impl InstructionBuilder for super::builders::Lock {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let mut accounts = vec![
            AccountMeta::new_readonly(self.approver, true),
            if let Some(token) = self.token {
                AccountMeta::new(token, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            AccountMeta::new_readonly(self.mint, false),
            AccountMeta::new(self.metadata, false),
            AccountMeta::new_readonly(self.edition.unwrap_or(crate::ID), false),
            if let Some(token_record) = self.token_record {
                AccountMeta::new(token_record, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            AccountMeta::new(self.payer, true),
            AccountMeta::new_readonly(self.system_program, false),
            AccountMeta::new_readonly(self.sysvar_instructions, false),
            AccountMeta::new_readonly(self.spl_token_program.unwrap_or(crate::ID), false),
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
            data: MetadataInstruction::Lock(self.args.clone())
                .try_to_vec()
                .unwrap(),
        }
    }
}

/// Unlocks an asset. For non-programmable assets, this will also thaw the token account.
///
/// # Accounts:
///
///   0. `[signer]` Token owner or delegate
///   1. `[writable, optional]` Delegate record account
///   2. `[writable, optional]` Token account
///   3. `[]` Mint account
///   4. `[writable]` Metadata account
///   5. `[optional]` Edition account
///   6. `[signer, writable]` Payer
///   7. `[]` System Program
///   8. `[]` Instructions sysvar account
///   9. `[optional]` SPL Token Program
///   10. `[optional]` Token Authorization Rules program
///   11. `[optional]` Token Authorization Rules account
impl InstructionBuilder for super::builders::Unlock {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let mut accounts = vec![
            AccountMeta::new_readonly(self.approver, true),
            if let Some(token) = self.token {
                AccountMeta::new(token, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            AccountMeta::new_readonly(self.mint, false),
            AccountMeta::new(self.metadata, false),
            AccountMeta::new_readonly(self.edition.unwrap_or(crate::ID), false),
            if let Some(token_record) = self.token_record {
                AccountMeta::new(token_record, false)
            } else {
                AccountMeta::new_readonly(crate::ID, false)
            },
            AccountMeta::new(self.payer, true),
            AccountMeta::new_readonly(self.system_program, false),
            AccountMeta::new_readonly(self.sysvar_instructions, false),
            AccountMeta::new_readonly(self.spl_token_program.unwrap_or(crate::ID), false),
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
            data: MetadataInstruction::Unlock(self.args.clone())
                .try_to_vec()
                .unwrap(),
        }
    }
}
