use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::cmp_pubkeys;
use num_derive::ToPrimitive;
#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};
use shank::ShankAccount;
use solana_program::program_option::COption;
use solana_program::program_pack::Pack;
use solana_program::{
    account_info::AccountInfo, instruction::AccountMeta, program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::state::Account;
use std::fmt;

use super::{Key, MetadataDelegateRecord, TokenMetadataAccount};
use crate::instruction::MetadataDelegateRole;
use crate::pda::{find_metadata_delegate_record_account, find_token_record_account};
use crate::utils::{assert_owned_by, try_from_slice_checked};

pub const TOKEN_RECORD_SEED: &str = "token_record";

pub const TOKEN_RECORD_SIZE: usize = 1 // Key
+ 1  // bump
+ 33 // delegate
+ 2  // delegate role
+ 1; // state

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
/// SEEDS = [
///     "metadata",
///     program id,
///     mint id,
///     "token_record",
///     token owner id
/// ]
pub struct TokenRecord {
    pub key: Key,
    pub bump: u8,
    pub delegate: Option<Pubkey>,
    pub delegate_role: Option<TokenDelegateRole>,
    pub state: TokenState,
}

impl Default for TokenRecord {
    fn default() -> Self {
        Self {
            key: Key::TokenRecord,
            bump: 255,
            delegate: None,
            delegate_role: None,
            state: TokenState::Unlocked,
        }
    }
}

impl TokenMetadataAccount for TokenRecord {
    fn key() -> Key {
        Key::TokenRecord
    }

    fn size() -> usize {
        TOKEN_RECORD_SIZE
    }
}

impl TokenRecord {
    pub fn from_bytes(data: &[u8]) -> Result<TokenRecord, ProgramError> {
        let record: TokenRecord =
            try_from_slice_checked(data, Key::TokenRecord, TokenRecord::size())?;
        Ok(record)
    }

    pub fn is_locked(&self) -> bool {
        matches!(self.state, TokenState::Locked)
    }
}

/// Programmable account state.
#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum TokenState {
    /// Token account is unlocked; operations are allowed on this account.
    Unlocked,
    /// Token account has been locked; no operations are allowed on this account.
    Locked,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum TokenDelegateRole {
    Sale,
    Transfer,
    Utility,
}

impl fmt::Display for TokenDelegateRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            Self::Sale => "sale_delegate".to_string(),
            Self::Transfer => "transfer_delegate".to_string(),
            Self::Utility => "use_delegate".to_string(),
        };

        write!(f, "{}", message)
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum AuthorityType {
    None,
    Metadata,
    Delegate,
    Holder,
}

pub struct AuthorityRequest<'a, 'b> {
    /// Pubkey of the authority.
    pub authority: &'a Pubkey,
    /// Metadata's update authority pubkey of the asset.
    pub update_authority: &'b Pubkey,
    /// Mint address.
    pub mint: &'a Pubkey,
    /// Holder's token account.
    pub token_info: Option<&'a AccountInfo<'a>>,
    /// `MetadataDelegateRecord` account of the authority (when the authority is a delegate).
    pub metadata_delegate_record_info: Option<&'a AccountInfo<'a>>,
    /// Expected `MetadataDelegateRole` for the request.
    pub metadata_delegate_role: Option<MetadataDelegateRole>,
    /// `TokenRecord` account.
    pub token_record_info: Option<&'a AccountInfo<'a>>,
    /// Expected `TokenDelegateRole` for the request.
    pub token_delegate_role: Option<TokenDelegateRole>,
}

impl AuthorityType {
    /// Determines the `AuthorityType`.
    pub fn get_authority_type(request: AuthorityRequest) -> Result<Self, ProgramError> {
        let token = if let Some(token_info) = request.token_info {
            Some(Account::unpack(&token_info.try_borrow_data()?)?)
        } else {
            None
        };

        // checks if the authority is the token owner

        if let Some(token) = token {
            if cmp_pubkeys(&token.owner, request.authority) {
                return Ok(AuthorityType::Holder);
            }
        }

        // checks if the authority is a token delegate

        if let Some(token_record_info) = request.token_record_info {
            // must be owned by token medatata
            assert_owned_by(token_record_info, &crate::ID)?;

            // we can only validate if it is a token delegate if we have the token account
            if let Some(token) = token {
                let (pda_key, _) = find_token_record_account(request.mint, &token.owner);
                let token_record = TokenRecord::from_account_info(token_record_info)?;

                if cmp_pubkeys(&pda_key, token_record_info.key)
                    && token_record.delegate_role == request.token_delegate_role
                    && (COption::from(token_record.delegate) == token.delegate)
                    && token.delegated_amount == token.amount
                {
                    return Ok(AuthorityType::Delegate);
                }
            }
        }

        // checks if the authority is a metadata delegate

        if let Some(metadata_delegate_record_info) = request.metadata_delegate_record_info {
            // must be owned by token medatata
            assert_owned_by(metadata_delegate_record_info, &crate::ID)?;

            if let Some(delegate_role) = request.metadata_delegate_role {
                let (pda_key, _) = find_metadata_delegate_record_account(
                    request.mint,
                    delegate_role,
                    request.update_authority,
                    request.authority,
                );

                if cmp_pubkeys(&pda_key, metadata_delegate_record_info.key) {
                    let delegate_record =
                        MetadataDelegateRecord::from_account_info(metadata_delegate_record_info)?;

                    if delegate_record.delegate == *request.authority {
                        return Ok(AuthorityType::Delegate);
                    }
                }
            }
        }

        // checks if the authority is the update authority

        if cmp_pubkeys(request.update_authority, request.authority) {
            return Ok(AuthorityType::Metadata);
        }

        Ok(AuthorityType::None)
    }
}

#[derive(Debug, Clone, ToPrimitive)]
pub enum Operation {
    Delegate,
    Transfer,
    Sale,
    MigrateClass,
    Update,
}

impl ToString for Operation {
    fn to_string(&self) -> String {
        match self {
            Operation::Delegate => "Delegate",
            Operation::Transfer => "Transfer",
            Operation::Sale => "Sale",
            Operation::MigrateClass => "MigrateClass",
            Operation::Update => "Update",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, ToPrimitive)]
pub enum PayloadKey {
    Amount,
    Authority,
    Destination,
    Holder,
    Delegate,
    Target,
}

impl ToString for PayloadKey {
    fn to_string(&self) -> String {
        match self {
            PayloadKey::Amount => "Amount",
            PayloadKey::Authority => "Authority",
            PayloadKey::Holder => "Holder",
            PayloadKey::Delegate => "Delegate",
            PayloadKey::Destination => "Destination",
            PayloadKey::Target => "Target",
        }
        .to_string()
    }
}

pub trait ToAccountMeta {
    fn to_account_meta(&self) -> AccountMeta;
}

impl<'info> ToAccountMeta for AccountInfo<'info> {
    fn to_account_meta(&self) -> AccountMeta {
        AccountMeta {
            pubkey: *self.key,
            is_signer: self.is_signer,
            is_writable: self.is_writable,
        }
    }
}
