use std::collections::{HashMap, HashSet};

use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;

use crate::{error::TrifleError, state::Key};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
pub struct EscrowConstraintModel {
    pub key: Key,
    pub creator: Pubkey,
    pub name: String,
    pub constraints: HashMap<String, EscrowConstraint>,
    pub update_authority: Pubkey,
    pub count: u64,
    pub schema_uri: Option<String>,
    pub royalties: RoyaltyModel,
    pub royalty_balance: u64,
}

impl EscrowConstraintModel {
    pub fn validate(&self, mint: &Pubkey, constraint_key: &String) -> Result<(), TrifleError> {
        if let Some(constraint) = self.constraints.get(constraint_key) {
            constraint.constraint_type.validate(mint)
        } else {
            Err(TrifleError::InvalidEscrowConstraint)
        }
    }
}

impl Default for EscrowConstraintModel {
    fn default() -> Self {
        Self {
            key: Key::EscrowConstraintModel,
            name: String::new(),
            constraints: HashMap::new(),
            creator: Pubkey::default(),
            update_authority: Pubkey::default(),
            count: 0,
            schema_uri: None,
            royalties: RoyaltyModel::default(),
            royalty_balance: 0,
        }
    }
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct EscrowConstraint {
    pub token_limit: u64,
    pub constraint_type: EscrowConstraintType,
    pub transfer_effects: u16,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum EscrowConstraintType {
    None,
    Collection(Pubkey),
    Tokens(HashSet<Pubkey>),
}

impl EscrowConstraintType {
    pub fn tokens_from_slice(tokens: &[Pubkey]) -> EscrowConstraintType {
        let mut h = HashSet::new();
        for token in tokens {
            h.insert(*token);
        }
        EscrowConstraintType::Tokens(h)
    }

    pub fn validate(&self, mint: &Pubkey) -> Result<(), TrifleError> {
        match self {
            EscrowConstraintType::None => Ok(()),
            EscrowConstraintType::Collection(collection) => {
                if collection == mint {
                    Ok(())
                } else {
                    Err(TrifleError::EscrowConstraintViolation)
                }
            }
            EscrowConstraintType::Tokens(tokens) => {
                if tokens.contains(mint) {
                    Ok(())
                } else {
                    Err(TrifleError::EscrowConstraintViolation)
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

pub const FEES: RoyaltyModel = RoyaltyModel {
    create_model: 694200,
    create_trifle: 694200,
    transfer_in: 694200,
    transfer_out: 694200,
    add_constraint: 694200,
};
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct RoyaltyModel {
    pub create_model: u64,
    pub create_trifle: u64,
    pub transfer_in: u64,
    pub transfer_out: u64,
    pub add_constraint: u64,
}
