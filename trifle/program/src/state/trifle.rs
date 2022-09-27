use crate::{
    error::TrifleError,
    state::{escrow_constraints::EscrowConstraintModel, Key},
};
use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;

use super::SolanaAccount;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
pub struct Trifle {
    pub key: Key,
    pub token_escrow: Pubkey,
    pub tokens: HashMap<String, Vec<TokenAmount>>,
    pub escrow_constraint_model: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct TokenAmount {
    pub mint: Pubkey,
    pub amount: u64,
}

impl TokenAmount {
    pub fn new(mint: Pubkey, amount: u64) -> Self {
        Self { mint, amount }
    }
}

// impl Trifle {
//     pub fn try_len(&self) -> Result<usize, ProgramError> {
//         Ok(1 + mem::size_of::<Pubkey>() + mem::size_of::<Pubkey>())
//     }
// }

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

impl Trifle {
    pub fn try_add(
        &mut self,
        constraint_model: &EscrowConstraintModel,
        constraint_key: String,
        token: Pubkey,
        amount: u64,
    ) -> Result<(), TrifleError> {
        constraint_model.validate(&token, &constraint_key)?;

        let constraint = constraint_model
            .constraints
            .get(&constraint_key)
            .ok_or(TrifleError::InvalidEscrowConstraint)?;

        let tokens = self.tokens.entry(constraint_key).or_insert(vec![]);

        // 0 means there is no limit to how many tokens may be added
        if constraint.token_limit != 0 {
            let current_amount = tokens.iter().try_fold(0u64, |acc, t| {
                acc.checked_add(t.amount)
                    .ok_or(TrifleError::NumericalOverflow)
            })?;

            if current_amount
                .checked_add(amount)
                .ok_or(TrifleError::NumericalOverflow)?
                > constraint.token_limit
            {
                return Err(TrifleError::TokenLimitExceeded);
            }
        }

        match tokens.iter_mut().find(|t| t.mint == token) {
            Some(t) => {
                t.amount = t
                    .amount
                    .checked_add(amount)
                    .ok_or(TrifleError::NumericalOverflow)?;
            }
            None => {
                tokens.push(TokenAmount::new(token, amount));
            }
        }

        Ok(())
    }
}

impl SolanaAccount for Trifle {
    fn key() -> Key {
        Key::Trifle
    }

    fn size() -> usize {
        0
    }
}
