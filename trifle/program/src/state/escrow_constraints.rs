use std::{
    collections::{HashMap, HashSet},
    mem,
};

use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::{error::TrifleError, state::Key};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
pub struct EscrowConstraintModel {
    pub key: Key,
    pub name: String,
    pub constraints: HashMap<String, EscrowConstraint>,
    pub creator: Pubkey,
    pub update_authority: Pubkey,
    pub count: u64,
}

impl EscrowConstraintModel {
    pub fn try_len(&self) -> Result<usize, ProgramError> {
        let map_overhead = 4;
        let string_overhead = 4;
        // let unknown_overhead = 8; // TODO: find out where this is coming from
        self.constraints
            .iter()
            .try_fold(0usize, |acc, (constraint_name, escrow_constraint)| {
                acc.checked_add(
                    escrow_constraint.try_len()? + constraint_name.len() + string_overhead,
                )
                .ok_or_else(|| TrifleError::NumericalOverflow.into())
            })
            .map(|ecs_len| {
                ecs_len
                    + 1 // key
                    + self.name.len()
                    + string_overhead // for name
                    + map_overhead // for constraints
                    + mem::size_of::<Pubkey>()
                    + mem::size_of::<Pubkey>()
                    + mem::size_of::<u64>()
            })
    }

    pub fn validate_at(&self, mint: &Pubkey, constraint_key: String) -> Result<(), ProgramError> {
        if let Some(constraint) = self.constraints.get(&constraint_key) {
            constraint.constraint_type.validate(mint)
        } else {
            Err(TrifleError::InvalidEscrowConstraintIndex.into())
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
        }
    }
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct EscrowConstraint {
    pub token_limit: u64,
    pub constraint_type: EscrowConstraintType,
}

impl EscrowConstraint {
    pub fn try_len(&self) -> Result<usize, ProgramError> {
        self.constraint_type
            .try_len()
            .map(|ct_len| Ok(ct_len + mem::size_of::<u64>()))?
    }
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum EscrowConstraintType {
    None,
    Collection(Pubkey),
    Tokens(HashSet<Pubkey>),
}

impl EscrowConstraintType {
    pub fn try_len(&self) -> Result<usize, ProgramError> {
        match self {
            EscrowConstraintType::None => Ok(1),
            EscrowConstraintType::Collection(_) => Ok(1 + mem::size_of::<Pubkey>()),
            EscrowConstraintType::Tokens(hm) => {
                if let Some(len) = hm.len().checked_mul(mem::size_of::<Pubkey>()) {
                    len.checked_add(1) // enum overhead
                        .ok_or(TrifleError::NumericalOverflow)?
                        .checked_add(4) // map overhead
                        .ok_or_else(|| TrifleError::NumericalOverflow.into())
                } else {
                    Err(TrifleError::NumericalOverflow.into())
                }
            }
        }
    }

    pub fn tokens_from_slice(tokens: &[Pubkey]) -> EscrowConstraintType {
        let mut hm = HashSet::new();
        for token in tokens {
            hm.insert(*token);
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
                    Err(TrifleError::EscrowConstraintViolation.into())
                }
            }
            EscrowConstraintType::Tokens(tokens) => {
                if tokens.contains(mint) {
                    Ok(())
                } else {
                    Err(TrifleError::EscrowConstraintViolation.into())
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
