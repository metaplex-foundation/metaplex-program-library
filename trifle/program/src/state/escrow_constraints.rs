use std::collections::{HashMap, HashSet};

use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;

use crate::{error::TrifleError, state::Key};

use super::SolanaAccount;

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
pub struct EscrowConstraintModel {
    /// The type of account
    pub key: Key,
    /// The creator of the Constraint Model
    pub creator: Pubkey,
    /// The Constraint Model name
    pub name: String,
    /// A map of constraints stored as Slot Name -> Constraint
    pub constraints: HashMap<String, EscrowConstraint>,
    /// The address with permission to modify this account
    pub update_authority: Pubkey,
    /// The number of times this Constraint Model is used, to prevent deletion of a used CM
    pub count: u64,
    /// Optional URI pointing to a schema for composing NFT metadata
    pub schema_uri: Option<String>,
    /// A list of creator royalties attached to Trifle
    /// This is stored as a map of Instruction -> royalty for granular control
    pub royalties: HashMap<RoyaltyInstruction, u64>,
    /// The current balance (in lamports) of paid fees stored in the Constraint Model account
    pub royalty_balance: u64,
    /// Buffer for future expansion
    pub padding: [u8; 32],
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
            royalties: HashMap::from([
                (RoyaltyInstruction::TransferIn, 0),
                (RoyaltyInstruction::TransferOut, 0),
            ]),
            royalty_balance: 0,
            padding: [0; 32],
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

// Protocol fees for Trifle
const TRIFLE_FEE: u64 = 20_000_000;
const MODEL_FEE: u64 = 100_000_000;

pub fn fees() -> HashMap<RoyaltyInstruction, u64> {
    let mut m = HashMap::new();
    m.insert(RoyaltyInstruction::CreateModel, 0);
    m.insert(RoyaltyInstruction::CreateTrifle, TRIFLE_FEE);
    m.insert(RoyaltyInstruction::TransferIn, TRIFLE_FEE);
    m.insert(RoyaltyInstruction::TransferOut, TRIFLE_FEE);
    m.insert(RoyaltyInstruction::AddConstraint, MODEL_FEE);
    m.insert(RoyaltyInstruction::RemoveConstraint, MODEL_FEE);
    m
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Hash, PartialOrd)]
pub enum RoyaltyInstruction {
    CreateModel,
    CreateTrifle,
    TransferIn,
    TransferOut,
    AddConstraint,
    RemoveConstraint,
}

impl SolanaAccount for EscrowConstraintModel {
    fn key() -> Key {
        Key::EscrowConstraintModel
    }

    fn size() -> usize {
        0
    }
}
