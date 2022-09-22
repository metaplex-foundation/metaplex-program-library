use crate::state::Key;
use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{program_error::ProgramError, pubkey::Pubkey};
use std::{collections::HashMap, mem};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
pub struct Trifle {
    pub key: Key,
    pub token_escrow: Pubkey,
    pub tokens: HashMap<usize, Vec<Pubkey>>,
    pub escrow_constraint_model: Pubkey,
}

impl Trifle {
    pub fn try_len(&self) -> Result<usize, ProgramError> {
        Ok(1 + mem::size_of::<Pubkey>() + mem::size_of::<Pubkey>())
    }
}

impl Default for Trifle {
    fn default() -> Self {
        Self {
            key: Key::Trifle,
            token_escrow: Pubkey::default(),
            tokens: HashMap::new(),
            escrow_constraint_model: Pubkey::default(),
        }
    }
}
