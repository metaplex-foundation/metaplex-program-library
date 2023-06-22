use std::io::Error;

use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::cmp_pubkeys;
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
    processor::{DelegateScenario, TransferScenario, UpdateScenario},
    utils::assert_owned_by,
};

/// Empty pubkey constant.
const DEFAULT_PUBKEY: Pubkey = Pubkey::new_from_array([0u8; 32]);

pub const TOKEN_RECORD_SEED: &str = "token_record";

pub const TOKEN_STATE_INDEX: usize = 2;

pub const LOCKED_TRANSFER_SIZE: usize = 33; // Optional Pubkey

pub const TOKEN_RECORD_SIZE: usize = 1 // Key
+ 1   // bump
+ 1   // state
+ 9   // rule set revision
+ 33  // delegate
+ 2   // delegate role
+ 33; // locked transfer

/// The `TokenRecord` struct represents the state of the token account holding a `pNFT`. Given
/// that the token account is always frozen, it includes a `state` that provides an abstraction
/// of frozen (locked) and thaw (unlocked).
///
/// It also stores state regarding token delegates that are set on the token account: the pubkey
/// of the delegate set (this would match the spl-token account delegate) and the role.
///
/// Every token account holding a `pNFT` has a token record associated. The seeds for the token
/// record PDA are:
/// 1. `"metadata"`
/// 2. program id
/// 3. mint id
/// 4. `"token_record"`
/// 5. token account id
#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
pub struct TokenRecord {
    /// Account key.
    pub key: Key,
    /// Derivation bump.
    pub bump: u8,
    /// Represented the token state.
    pub state: TokenState,
    /// Stores the rule set revision (if any). The revision is updated every time
    /// a new token delegate is approved.
    pub rule_set_revision: Option<u64>,
    /// Pubkey of the current token delegate. This delegate key will match the spl-token
    /// delegate pubkey.
    #[cfg_attr(
        feature = "serde-feature",
        serde(
            deserialize_with = "deser_option_pubkey",
            serialize_with = "ser_option_pubkey"
        )
    )]
    pub delegate: Option<Pubkey>,
    /// The role of the current token delegate.
    pub delegate_role: Option<TokenDelegateRole>,
    /// Stores the destination pubkey when a transfer is lock to an allowed address. This
    /// pubkey gets set when a 'LockTransfer' delegate is approved.
    pub locked_transfer: Option<Pubkey>,
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
            locked_transfer: None,
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

    fn safe_deserialize(data: &[u8]) -> Result<Self, BorshError> {
        Self::from_bytes(data).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
    }

    fn from_account_info(account_info: &AccountInfo) -> Result<Self, ProgramError> {
        let data = &account_info.try_borrow_data()?;
        Self::from_bytes(data)
    }
}

impl TokenRecord {
    pub fn is_locked(&self) -> bool {
        matches!(self.state, TokenState::Locked)
    }

    /// Resets the token state by clearing any state stored.
    pub fn reset(&mut self) {
        self.state = TokenState::Unlocked;
        self.rule_set_revision = None;
        self.delegate = None;
        self.delegate_role = None;
        self.locked_transfer = None;
    }
}

impl Resizable for TokenRecord {
    fn from_bytes<'a>(account_data: &[u8]) -> Result<TokenRecord, ProgramError> {
        // we perform a manual deserialization since we are potentially dealing
        // with accounts of different sizes
        let length = TokenRecord::size() as i64 - account_data.len() as i64;

        // we use the account length in the 'is_correct_account_type' since we are
        // manually checking that the account length is valid
        if !(length == 0 || length == LOCKED_TRANSFER_SIZE as i64)
            || !TokenRecord::is_correct_account_type(
                account_data,
                Key::TokenRecord,
                account_data.len(),
            )
        {
            return Err(MetadataError::DataTypeMismatch.into());
        }
        // mutable "pointer" to the account data
        let mut data = account_data;

        let key: Key = BorshDeserialize::deserialize(&mut data)?;
        let bump: u8 = BorshDeserialize::deserialize(&mut data)?;
        let state: TokenState = BorshDeserialize::deserialize(&mut data)?;
        let rule_set_revision: Option<u64> = BorshDeserialize::deserialize(&mut data)?;
        let delegate: Option<Pubkey> = BorshDeserialize::deserialize(&mut data)?;
        let delegate_role: Option<TokenDelegateRole> = BorshDeserialize::deserialize(&mut data)?;

        let locked_transfer: Option<Pubkey> = if length == 0 {
            BorshDeserialize::deserialize(&mut data)?
        } else {
            None
        };

        Ok(TokenRecord {
            key,
            bump,
            state,
            rule_set_revision,
            delegate,
            delegate_role,
            locked_transfer,
        })
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
    LockedTransfer,
    Migration = 255,
}

pub struct AuthorityRequest<'a, 'b> {
    /// Determines the precedence of authority types.
    pub precedence: &'a [AuthorityType],
    /// Pubkey of the authority.
    pub authority: &'a Pubkey,
    /// Metadata's update authority pubkey of the asset.
    pub update_authority: &'b Pubkey,
    /// Mint address.
    pub mint: &'b Pubkey,
    /// Collection mint address.
    pub collection_mint: Option<&'b Pubkey>,
    /// Holder's token account info.
    pub token: Option<&'a Pubkey>,
    /// Holder's token account.
    pub token_account: Option<&'b Account>,
    /// `MetadataDelegateRecord` account of the authority (when the authority is a delegate).
    pub metadata_delegate_record_info: Option<&'a AccountInfo<'a>>,
    /// Expected `MetadataDelegateRole` for the request.
    pub metadata_delegate_roles: Vec<MetadataDelegateRole>,
    /// Expected collection-level `MetadataDelegateRole` for the request.
    pub collection_metadata_delegate_roles: Vec<MetadataDelegateRole>,
    /// `TokenRecord` account.
    pub token_record_info: Option<&'a AccountInfo<'a>>,
    /// Expected `TokenDelegateRole` for the request.
    pub token_delegate_roles: Vec<TokenDelegateRole>,
}

impl<'a, 'b> Default for AuthorityRequest<'a, 'b> {
    fn default() -> Self {
        Self {
            precedence: &[
                AuthorityType::TokenDelegate,
                AuthorityType::Holder,
                AuthorityType::MetadataDelegate,
                AuthorityType::Metadata,
            ],
            authority: &DEFAULT_PUBKEY,
            update_authority: &DEFAULT_PUBKEY,
            mint: &DEFAULT_PUBKEY,
            collection_mint: None,
            token: None,
            token_account: None,
            metadata_delegate_record_info: None,
            metadata_delegate_roles: Vec::with_capacity(0),
            collection_metadata_delegate_roles: Vec::with_capacity(0),
            token_record_info: None,
            token_delegate_roles: Vec::with_capacity(0),
        }
    }
}

/// Struct to represent the authority type identified from
/// an authority request.
#[derive(Default)]
pub struct AuthorityResponse {
    pub authority_type: AuthorityType,
    pub token_delegate_role: Option<TokenDelegateRole>,
    pub metadata_delegate_role: Option<MetadataDelegateRole>,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
pub enum AuthorityType {
    #[default]
    None,
    Metadata,
    Holder,
    MetadataDelegate,
    TokenDelegate,
}

impl AuthorityType {
    /// Determines the `AuthorityType`.
    ///
    /// The `AuthorityType` is used to determine the authority of a request. An authority can
    /// be "valid" for multiples types (e.g., the same authority can be the holder and the update
    /// authority). This ambiguity is resolved by using the `precedence`, which determines the
    /// priority of types.
    pub fn get_authority_type(
        request: AuthorityRequest,
    ) -> Result<AuthorityResponse, ProgramError> {
        // the evaluation follows the `request.precedence` order; as soon as a match is
        // found, the type is returned
        for authority_type in request.precedence {
            match authority_type {
                AuthorityType::TokenDelegate => {
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
                                return Ok(AuthorityResponse {
                                    authority_type: AuthorityType::TokenDelegate,
                                    token_delegate_role: token_record.delegate_role,
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
                AuthorityType::MetadataDelegate => {
                    // checks if the authority is a metadata delegate

                    if let Some(metadata_delegate_record_info) =
                        request.metadata_delegate_record_info
                    {
                        // must be owned by token medatata
                        assert_owned_by(metadata_delegate_record_info, &crate::ID)?;

                        for role in &request.metadata_delegate_roles {
                            // looking up the delegate on the metadata mint
                            let (pda_key, _) = find_metadata_delegate_record_account(
                                request.mint,
                                *role,
                                request.update_authority,
                                request.authority,
                            );

                            if cmp_pubkeys(&pda_key, metadata_delegate_record_info.key) {
                                let delegate_record = MetadataDelegateRecord::from_account_info(
                                    metadata_delegate_record_info,
                                )?;

                                if delegate_record.delegate == *request.authority {
                                    return Ok(AuthorityResponse {
                                        authority_type: AuthorityType::MetadataDelegate,
                                        metadata_delegate_role: Some(*role),
                                        ..Default::default()
                                    });
                                }
                            }
                        }

                        // looking up the delegate on the collection mint (this is for
                        // collection-level delegates)
                        if let Some(collection_mint) = request.collection_mint {
                            for role in &request.collection_metadata_delegate_roles {
                                let (pda_key, _) = find_metadata_delegate_record_account(
                                    collection_mint,
                                    *role,
                                    request.update_authority,
                                    request.authority,
                                );

                                if cmp_pubkeys(&pda_key, metadata_delegate_record_info.key) {
                                    let delegate_record =
                                        MetadataDelegateRecord::from_account_info(
                                            metadata_delegate_record_info,
                                        )?;

                                    if delegate_record.delegate == *request.authority {
                                        return Ok(AuthorityResponse {
                                            authority_type: AuthorityType::MetadataDelegate,
                                            metadata_delegate_role: Some(*role),
                                            ..Default::default()
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                AuthorityType::Holder => {
                    // checks if the authority is the token owner

                    if let Some(token_account) = request.token_account {
                        if cmp_pubkeys(&token_account.owner, request.authority) {
                            return Ok(AuthorityResponse {
                                authority_type: AuthorityType::Holder,
                                ..Default::default()
                            });
                        }
                    }
                }
                AuthorityType::Metadata => {
                    // checks if the authority is the update authority

                    if cmp_pubkeys(request.update_authority, request.authority) {
                        return Ok(AuthorityResponse {
                            authority_type: AuthorityType::Metadata,
                            ..Default::default()
                        });
                    }
                }
                _ => { /* the default return type is 'None' */ }
            }
        }

        // if we reach this point, no 'valid' authority type has been found
        Ok(AuthorityResponse::default())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    Transfer { scenario: TransferScenario },
    Update { scenario: UpdateScenario },
    Delegate { scenario: DelegateScenario },
}

impl ToString for Operation {
    fn to_string(&self) -> String {
        match self {
            Self::Transfer { scenario } => format!("Transfer:{}", scenario),
            Self::Update { scenario } => format!("Update:{}", scenario),
            Self::Delegate { scenario } => format!("Delegate:{}", scenario),
        }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
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
