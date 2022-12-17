use std::fmt;

use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

use crate::instruction::MetadataInstruction;

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum DelegateArgs {
    CollectionV1,
    TransferV1 { amount: u64 },
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum RevokeArgs {
    CollectionV1,
    TransferV1,
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
}

impl fmt::Display for DelegateRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            Self::Authority => "authority_delegate".to_string(),
            Self::Collection => "collection_delegate".to_string(),
            Self::Transfer => "sale_delegate".to_string(),
            Self::Use => "use_delegate".to_string(),
            Self::Utility => "utility_delegate".to_string(),
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
///   2. `[]` Mint account
///   3. `[writable]` Metadata account
///   4. `[optional]` Master Edition account
///   5. `[signer]` Authority to approve the delegation
///   6. `[signer, writable]` Payer
///   7. `[]` System Program
///   8. `[]` Instructions sysvar account
///   9. `[optional]` SPL Token Program
///   10. `[optional, writable]` Token account
///   11. `[optional]` Token Authorization Rules program
///   12. `[optional]` Token Authorization Rules account
pub fn delegate(
    delegate: Pubkey,
    delegate_owner: Pubkey,
    mint: Pubkey,
    metadata: Pubkey,
    master_edition: Option<Pubkey>,
    authority: Pubkey,
    payer: Pubkey,
    token: Option<Pubkey>,
    authorization_rules: Option<Pubkey>,
    additional_accounts: Option<Vec<AccountMeta>>,
    args: DelegateArgs,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(delegate, false),
        AccountMeta::new_readonly(delegate_owner, false),
        AccountMeta::new_readonly(mint, false),
        AccountMeta::new(metadata, false),
        AccountMeta::new_readonly(
            if let Some(master_edition) = master_edition {
                master_edition
            } else {
                crate::ID
            },
            false,
        ),
        AccountMeta::new_readonly(authority, true),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::instructions::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new(
            if let Some(token) = token {
                token
            } else {
                crate::id()
            },
            false,
        ),
    ];

    if let Some(authorization_rules) = authorization_rules {
        accounts.push(AccountMeta::new_readonly(mpl_token_auth_rules::id(), false));
        accounts.push(AccountMeta::new_readonly(authorization_rules, false));
    } else {
        accounts.push(AccountMeta::new_readonly(crate::id(), false));
        accounts.push(AccountMeta::new_readonly(crate::id(), false));
    }

    if let Some(additional_accounts) = additional_accounts {
        accounts.extend(additional_accounts);
    }

    Instruction {
        program_id: crate::id(),
        accounts,
        data: MetadataInstruction::Delegate(args).try_to_vec().unwrap(),
    }
}

/// Revoke a delegation of the token.
///
/// # Accounts:
///
///   0. `[writable]` Delegate account key
///   1. `[]` Delegated owner
///   2. `[]` Mint account
///   3. `[writable]` Metadata account
///   4. `[optional]` Master Edition account
///   5. `[signer]` Authority to approve the delegation
///   6. `[signer, writable]` Payer
///   7. `[]` System Program
///   8. `[]` Instructions sysvar account
///   9. `[optional]` SPL Token Program
///   10. `[optional, writable]` Token account
///   11. `[optional]` Token Authorization Rules program
///   12. `[optional]` Token Authorization Rules account
pub fn revoke(
    delegate: Pubkey,
    delegate_owner: Pubkey,
    mint: Pubkey,
    metadata: Pubkey,
    master_edition: Option<Pubkey>,
    authority: Pubkey,
    payer: Pubkey,
    token: Option<Pubkey>,
    authorization_rules: Option<Pubkey>,
    additional_accounts: Option<Vec<AccountMeta>>,
    args: RevokeArgs,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(delegate, false),
        AccountMeta::new_readonly(delegate_owner, false),
        AccountMeta::new_readonly(mint, false),
        AccountMeta::new(metadata, false),
        AccountMeta::new_readonly(
            if let Some(master_edition) = master_edition {
                master_edition
            } else {
                crate::id()
            },
            false,
        ),
        AccountMeta::new_readonly(authority, true),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::instructions::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new(
            if let Some(token) = token {
                token
            } else {
                crate::id()
            },
            false,
        ),
    ];

    if let Some(authorization_rules) = authorization_rules {
        accounts.push(AccountMeta::new_readonly(mpl_token_auth_rules::id(), false));
        accounts.push(AccountMeta::new_readonly(authorization_rules, false));
    } else {
        accounts.push(AccountMeta::new_readonly(crate::id(), false));
        accounts.push(AccountMeta::new_readonly(crate::id(), false));
    }

    if let Some(additional_accounts) = additional_accounts {
        accounts.extend(additional_accounts);
    }

    Instruction {
        program_id: crate::id(),
        accounts,
        data: MetadataInstruction::Revoke(args).try_to_vec().unwrap(),
    }
}
