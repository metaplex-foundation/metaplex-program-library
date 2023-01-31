use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::cmp_pubkeys;
use num_derive::ToPrimitive;
#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, instruction::AccountMeta, program_error::ProgramError,
    program_option::COption, pubkey::Pubkey,
};
use spl_token::state::Account;

use super::*;
use crate::{
    error::MetadataError,
    instruction::MetadataDelegateRole,
    pda::{find_metadata_delegate_record_account, find_token_record_account},
    processor::{TransferScenario, UpdateScenario},
    state::BorshError,
    utils::{assert_owned_by, try_from_slice_checked},
};

/// Empty pubkey constant.
const DEFAULT_PUBKEY: Pubkey = Pubkey::new_from_array([0u8; 32]);

pub const TOKEN_RECORD_SEED: &str = "token_record";

pub const TOKEN_STATE_INDEX: usize = 2;

pub const TOKEN_RECORD_SIZE: usize = 1 // Key
+ 1  // bump
+ 1  // state
+ 9  // rule set revision
+ 33 // delegate
+ 2; // delegate role

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
    pub state: TokenState,
    pub rule_set_revision: Option<u64>,
    #[cfg_attr(
        feature = "serde-feature",
        serde(
            deserialize_with = "deser_option_pubkey",
            serialize_with = "ser_option_pubkey"
        )
    )]
    pub delegate: Option<Pubkey>,
    pub delegate_role: Option<TokenDelegateRole>,
}

impl Default for TokenRecord {
    fn default() -> Self {
        Self {
            key: Key::TokenRecord,
            bump: 255,
            state: TokenState::Unlocked,
            rule_set_revision: None,
            delegate: None,
            delegate_role: None,
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

    /// Resets the token state by clearing any state stored.
    pub fn reset(&mut self) {
        self.state = TokenState::Unlocked;
        self.rule_set_revision = None;
        self.delegate = None;
        self.delegate_role = None;
    }

    pub fn save(&self, data: &mut [u8]) -> Result<(), BorshError> {
        let mut storage = &mut data[..Self::size()];
        BorshSerialize::serialize(self, &mut storage)?;
        Ok(())
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
    /// Token account has a `Sale` delegate set; operations are restricted.
    Listed,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum TokenDelegateRole {
    Sale,
    Transfer,
    Utility,
    Staking,
    Standard,
    Migration = 255,
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
    /// Determines the precedence of authority types.
    pub precedence: &'a [AuthorityType],
    /// Pubkey of the authority.
    pub authority: &'a Pubkey,
    /// Metadata's update authority pubkey of the asset.
    pub update_authority: &'b Pubkey,
    /// Mint address.
    pub mint: &'a Pubkey,
    /// Holder's token account info.
    pub token: Option<&'a Pubkey>,
    /// Holder's token account.
    pub token_account: Option<&'b Account>,
    /// `MetadataDelegateRecord` account of the authority (when the authority is a delegate).
    pub metadata_delegate_record_info: Option<&'a AccountInfo<'a>>,
    /// Expected `MetadataDelegateRole` for the request.
    pub metadata_delegate_role: Option<MetadataDelegateRole>,
    /// `TokenRecord` account.
    pub token_record_info: Option<&'a AccountInfo<'a>>,
    /// Expected `TokenDelegateRole` for the request.
    pub token_delegate_roles: Vec<TokenDelegateRole>,
}

impl<'a, 'b> Default for AuthorityRequest<'a, 'b> {
    fn default() -> Self {
        Self {
            precedence: &[
                AuthorityType::Delegate,
                AuthorityType::Holder,
                AuthorityType::Metadata,
            ],
            authority: &DEFAULT_PUBKEY,
            update_authority: &DEFAULT_PUBKEY,
            mint: &DEFAULT_PUBKEY,
            token: None,
            token_account: None,
            metadata_delegate_record_info: None,
            metadata_delegate_role: None,
            token_record_info: None,
            token_delegate_roles: Vec::with_capacity(0),
        }
    }
}

impl AuthorityType {
    /// Determines the `AuthorityType`.
    ///
    /// The `AuthorityType` is used to determine the authority of a request. An authority can
    /// be "valid" for multiples types (e.g., the same authority can be the holder and the update
    /// authority). This ambiguity is resolved by using the `precedence`, which determines the
    /// priority of types.
    pub fn get_authority_type(request: AuthorityRequest) -> Result<Self, ProgramError> {
        // the evaluation follows the `request.precedence` order; as soon as a match is
        // found, the type is returned
        for authority_type in request.precedence {
            match authority_type {
                AuthorityType::Delegate => {
                    // checks if the authority is a token delegate

                    if let Some(token_record_info) = request.token_record_info {
                        // must be owned by token medatata
                        assert_owned_by(token_record_info, &crate::ID)?;

                        // we can only validate if it is a token delegate when we have the token account
                        if let Some(token_account) = request.token_account {
                            let token = request.token.ok_or(MetadataError::MissingTokenAccount)?;

                            let (pda_key, _) = find_token_record_account(request.mint, token);
                            let token_record = TokenRecord::from_account_info(token_record_info)?;

                            let role_matches = match token_record.delegate_role {
                                Some(role) => request.token_delegate_roles.contains(&role),
                                None => request.token_delegate_roles.is_empty(),
                            };

                            if cmp_pubkeys(&pda_key, token_record_info.key)
                                && Some(*request.authority) == token_record.delegate
                                && role_matches
                                && (COption::from(token_record.delegate) == token_account.delegate)
                            {
                                return Ok(AuthorityType::Delegate);
                            }
                        }
                    }

                    // checks if the authority is a metadata delegate

                    if let Some(metadata_delegate_record_info) =
                        request.metadata_delegate_record_info
                    {
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
                                let delegate_record = MetadataDelegateRecord::from_account_info(
                                    metadata_delegate_record_info,
                                )?;

                                if delegate_record.delegate == *request.authority {
                                    return Ok(AuthorityType::Delegate);
                                }
                            }
                        }
                    }
                }
                AuthorityType::Holder => {
                    // checks if the authority is the token owner

                    if let Some(token_account) = request.token_account {
                        if cmp_pubkeys(&token_account.owner, request.authority) {
                            return Ok(AuthorityType::Holder);
                        }
                    }
                }
                AuthorityType::Metadata => {
                    // checks if the authority is the update authority

                    if cmp_pubkeys(request.update_authority, request.authority) {
                        return Ok(AuthorityType::Metadata);
                    }
                }
                _ => { /* the default return type is 'None' */ }
            }
        }

        // if we reach this point, no 'valid' authority type has been found
        Ok(AuthorityType::None)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    Transfer { scenario: TransferScenario },
    Update { scenario: UpdateScenario },
}

impl ToString for Operation {
    fn to_string(&self) -> String {
        match self {
            Self::Transfer { scenario } => format!("Transfer:{}", scenario),
            Self::Update { scenario } => format!("Update:{}", scenario),
        }
    }
}

#[derive(Debug, Clone, ToPrimitive)]
pub enum PayloadKey {
    Amount,
    Authority,
    AuthoritySeeds,
    Delegate,
    DelegateSeeds,
    Destination,
    DestinationSeeds,
    Holder,
    Source,
    SourceSeeds,
}

impl ToString for PayloadKey {
    fn to_string(&self) -> String {
        match self {
            PayloadKey::Amount => "Amount",
            PayloadKey::Authority => "Authority",
            PayloadKey::AuthoritySeeds => "AuthoritySeeds",
            PayloadKey::Delegate => "Delegate",
            PayloadKey::DelegateSeeds => "DelegateSeeds",
            PayloadKey::Destination => "Destination",
            PayloadKey::DestinationSeeds => "DestinationSeeds",
            PayloadKey::Holder => "Holder",
            PayloadKey::Source => "Source",
            PayloadKey::SourceSeeds => "SourceSeeds",
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
