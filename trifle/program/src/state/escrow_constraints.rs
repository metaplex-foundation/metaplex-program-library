use std::{collections::HashSet, mem};

use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::{error::TrifleError, state::Key};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
pub struct EscrowConstraintModel {
    pub key: Key,
    pub name: String,
    pub constraints: Vec<EscrowConstraint>,
    pub creator: Pubkey,
    pub update_authority: Pubkey,
    pub count: u64,
}

impl EscrowConstraintModel {
    pub fn try_len(&self) -> Result<usize, ProgramError> {
        let unknown_overhead = 8; // TODO: find out where this is coming from
        self.constraints
            .iter()
            .try_fold(0usize, |acc, ec| {
                acc.checked_add(ec.try_len()?)
                    .ok_or_else(|| TrifleError::NumericalOverflow.into())
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
            Err(TrifleError::InvalidEscrowConstraintIndex.into())
        }
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

#[repr(C)]
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
