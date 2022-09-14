use std::{collections::HashMap, mem};

use crate::{
    error::MetadataError,
    state::{Key, TokenMetadataAccount},
};
use borsh::{BorshDeserialize, BorshSerialize};
use num_traits::FromPrimitive;
use shank::ShankAccount;
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};

pub const ESCROW_PREFIX: &str = "escrow";

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
pub struct TokenOwnedEscrow {
    pub key: Key,
    pub base_token: Pubkey,
    pub tokens: Vec<Option<Pubkey>>,
    pub delegates: Vec<Pubkey>,
    pub model: Option<Pubkey>,
}

impl TokenOwnedEscrow {
    pub fn len(&self) -> usize {
        let mut len = mem::size_of::<Key>();
        len += mem::size_of::<Pubkey>();
        len += 4 + self.tokens.len() * mem::size_of::<Option<Pubkey>>();
        len += 4 + self.delegates.len() * mem::size_of::<Pubkey>();
        len += mem::size_of::<Option<Pubkey>>();
        len
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty() && self.delegates.is_empty() && self.model.is_none()
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct EscrowConstraintModel {
    pub key: Key,
    pub name: String,
    pub constraints: Vec<EscrowConstraint>,
    pub creator: Pubkey,
    pub update_authority: Pubkey,
    /// The number of Token Owned Escrow accounts that are using this model.
    pub count: u64,
}

impl EscrowConstraintModel {
    pub fn try_len(&self) -> Result<usize, ProgramError> {
        let unknown_overhead = 8; // TODO: find out where this is coming from
        self.constraints
            .iter()
            .try_fold(0usize, |acc, ec| {
                acc.checked_add(ec.try_len()?)
                    .ok_or_else(|| MetadataError::NumericalOverflowError.into())
            })
            .map(|ecs_len| {
                ecs_len
                    + 1 // key
                    + self.name.len()
                    + mem::size_of::<Pubkey>()
                    + mem::size_of::<Pubkey>()
                    + mem::size_of::<u64>()
                    + unknown_overhead
            })
    }

    pub fn validate_at(&self, mint: &Pubkey, index: usize) -> Result<(), ProgramError> {
        if let Some(constraint) = self.constraints.get(index) {
            constraint.constraint_type.validate(mint)
        } else {
            Err(MetadataError::InvalidEscrowConstraintIndex.into())
        }
        // self.constraints
        //     .get(index)
        //     .ok_or::<MetadataError>(MetadataError::InvalidEscrowConstraintIndex.into())?
        //     .constraint_type
        //     .validate(mint)
    }
}

impl Default for EscrowConstraintModel {
    fn default() -> Self {
        Self {
            key: Key::EscrowConstraintModel,
            name: String::new(),
            constraints: vec![],
            creator: Pubkey::default(),
            update_authority: Pubkey::default(),
            count: 0,
        }
    }
}

// impl EscrowConstraintModelAccount for EscrowConstraintModel {}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct EscrowConstraint {
    pub name: String,
    pub token_limit: u64,
    pub constraint_type: EscrowConstraintType,
}

impl EscrowConstraint {
    pub fn try_len(&self) -> Result<usize, ProgramError> {
        let unknown_overhead = 4; // TODO: find out where this is coming from
        self.constraint_type
            .try_len()
            .map(|ct_len| Ok(ct_len + self.name.len() + mem::size_of::<u64>() + unknown_overhead))?
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum EscrowConstraintType {
    None,
    Collection(Pubkey),
    Tokens(HashMap<Pubkey, ()>),
}

impl EscrowConstraintType {
    pub fn try_len(&self) -> Result<usize, ProgramError> {
        match self {
            EscrowConstraintType::None => Ok(1),
            EscrowConstraintType::Collection(_) => Ok(1 + mem::size_of::<Pubkey>()),
            EscrowConstraintType::Tokens(hm) => {
                if let Some(len) = hm.len().checked_mul(mem::size_of::<Pubkey>()) {
                    len.checked_add(1) // enum overhead
                        .ok_or(MetadataError::NumericalOverflowError)?
                        .checked_add(4) // map overhead
                        .ok_or_else(|| MetadataError::NumericalOverflowError.into())
                } else {
                    Err(MetadataError::NumericalOverflowError.into())
                }
            }
        }
    }

    pub fn tokens_from_slice(tokens: &[Pubkey]) -> EscrowConstraintType {
        let mut hm = HashMap::new();
        for token in tokens {
            hm.insert(*token, ());
        }
        EscrowConstraintType::Tokens(hm)
    }

    pub fn validate(&self, mint: &Pubkey) -> Result<(), ProgramError> {
        match self {
            EscrowConstraintType::None => Ok(()),
            EscrowConstraintType::Collection(collection) => {
                if collection == mint {
                    Ok(())
                } else {
                    Err(MetadataError::EscrowConstraintViolation.into())
                }
            }
            EscrowConstraintType::Tokens(tokens) => {
                if tokens.contains_key(mint) {
                    Ok(())
                } else {
                    Err(MetadataError::EscrowConstraintViolation.into())
                }
            }
        }
    }
}

impl Default for EscrowConstraintType {
    fn default() -> Self {
        EscrowConstraintType::None
    }
}

impl TokenMetadataAccount for TokenOwnedEscrow {
    fn key() -> Key {
        Key::TokenOwnedEscrow
    }

    fn size() -> usize {
        0
    }

    fn is_correct_account_type(data: &[u8], data_type: Key, _data_size: usize) -> bool {
        let key: Option<Key> = Key::from_u8(data[0]);
        match key {
            Some(key) => (key == data_type || key == Key::Uninitialized),
            None => false,
        }
    }
}

impl TokenMetadataAccount for EscrowConstraintModel {
    fn key() -> Key {
        Key::EscrowConstraintModel
    }

    fn size() -> usize {
        0
    }

    fn is_correct_account_type(data: &[u8], data_type: Key, _data_size: usize) -> bool {
        let key: Option<Key> = Key::from_u8(data[0]);
        match key {
            Some(key) => (key == data_type || key == Key::Uninitialized),
            None => false,
        }
    }
}
