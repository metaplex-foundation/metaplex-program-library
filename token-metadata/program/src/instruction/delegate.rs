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
    V1 { role: DelegateRole },
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum RevokeArgs {
    V1 { role: DelegateRole },
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum DelegateRole {
    Authority,
    Collection,
    Sale,
    Use,
    Utility,
}

impl fmt::Display for DelegateRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            Self::Authority => "authority_delegate".to_string(),
            Self::Collection => "collection_delegate".to_string(),
            Self::Sale => "sale_delegate".to_string(),
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
///   2. `[signer]` Owner
///   3. `[signer, writable]` Payer
///   4. `[writable]` Token account
///   5. `[writable]` Metadata account
///   6. `[]` Mint account
///   7. `[]` System Program
///   8. `[]` Instructions sysvar account
///   9. `[]` SPL Token Program
///   10. `[optional]` Token Authorization Rules account
///   11. `[optional]` Token Authorization Rules program
pub fn delegate(
    delegate: Pubkey,
    delegate_owner: Pubkey,
    owner: Pubkey,
    payer: Pubkey,
    token: Pubkey,
    metadata: Pubkey,
    mint: Pubkey,
    role: DelegateRole,
    authorization_rules: Option<Pubkey>,
    additional_accounts: Option<Vec<AccountMeta>>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(delegate, false),
        AccountMeta::new_readonly(delegate_owner, false),
        AccountMeta::new_readonly(owner, true),
        AccountMeta::new(payer, true),
        AccountMeta::new(token, false),
        AccountMeta::new(metadata, false),
        AccountMeta::new_readonly(mint, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::instructions::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    if let Some(authorization_rules) = authorization_rules {
        accounts.push(AccountMeta::new_readonly(authorization_rules, false));
        //accounts.push(AccountMeta::new_readonly(token_authorization::id(), false));
    }

    if let Some(additional_accounts) = additional_accounts {
        accounts.extend(additional_accounts);
    }

    Instruction {
        program_id: crate::id(),
        accounts,
        data: MetadataInstruction::Delegate(DelegateArgs::V1 { role })
            .try_to_vec()
            .unwrap(),
    }
}

/// Delegates the token.
///
/// # Accounts:
///
///   0. `[writable]` Delegate account key
///   1. `[]` Delegated user
///   2. `[signer]` Token owner
///   3. `[signer, writable]` Payer
///   4. `[writable]` Owned token account of mint
///   5. `[writable]` Metadata account
///   6. `[]` Mint of metadata
///   7. `[]` System Program
///   8. `[]` SPL Token Program
///   9. `[]` Instructions sysvar account
///   10. `[optional]` Token Authorization Rules account
///   11. `[optional]` Token Authorization Rules Program
pub fn revoke(
    program_id: Pubkey,
    delegate: Pubkey,
    user: Pubkey,
    token_owner: Pubkey,
    payer: Pubkey,
    token: Pubkey,
    metadata: Pubkey,
    args: RevokeArgs,
    authorization_rules: Option<Pubkey>,
    additional_accounts: Option<Vec<AccountMeta>>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(delegate, false),
        AccountMeta::new_readonly(user, false),
        AccountMeta::new_readonly(token_owner, true),
        AccountMeta::new(payer, true),
        AccountMeta::new(token, false),
        AccountMeta::new(metadata, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(sysvar::instructions::id(), false),
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
        data: MetadataInstruction::Revoke(args).try_to_vec().unwrap(),
    }
}
