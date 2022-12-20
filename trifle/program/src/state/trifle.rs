use crate::{error::TrifleError, state::Key};
use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;

use super::{escrow_constraints::EscrowConstraint, SolanaAccount};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
pub struct Trifle {
    /// The type of account
    pub key: Key,
    /// The pubkey of the COE on the parent NFT's metadata account
    pub token_escrow: Pubkey,
    /// A map of the Slot:Token pairs. Each slot can contain multiple token mints and amounts
    pub tokens: HashMap<String, Vec<TokenAmount>>,
    /// The pubkey of the Escrow Constraint Model
    pub escrow_constraint_model: Pubkey,
    /// Buffer for future expansion
    pub padding: [u8; 32],
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

impl Default for Trifle {
    fn default() -> Self {
        Self {
            key: Key::Trifle,
            token_escrow: Pubkey::default(),
            tokens: HashMap::new(),
            escrow_constraint_model: Pubkey::default(),
            padding: [0; 32],
        }
    }
}

impl Trifle {
    pub fn try_add(
        &mut self,
        constraint: &EscrowConstraint,
        constraint_key: String,
        token: Pubkey,
        amount: u64,
    ) -> Result<(), TrifleError> {
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

    pub fn try_remove(
        &mut self,
        constraint_key: String,
        mint: Pubkey,
        amount: u64,
    ) -> Result<(), TrifleError> {
        // find the constraint key, error if it doesn't exist
        let mut token_amounts = self
            .tokens
            .remove(&constraint_key)
            .ok_or(TrifleError::ConstraintKeyNotFound)?;

        let index = token_amounts
            .iter()
            .position(|t| t.mint == mint)
            .ok_or(TrifleError::FailedToFindTokenAmount)?;

        let mut token_amount = token_amounts.swap_remove(index);

        // subtract the amount from the token amount
        token_amount.amount = token_amount
            .amount
            .checked_sub(amount)
            .ok_or(TrifleError::NumericalOverflow)?;

        if token_amount.amount > 0 {
            token_amounts.push(token_amount);
        }
        if !token_amounts.is_empty() {
            self.tokens.insert(constraint_key, token_amounts);
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
            || self.tokens.iter().all(|(_, token_amounts)| {
                token_amounts.is_empty() || token_amounts.iter().all(|t| t.amount == 0)
            })
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
