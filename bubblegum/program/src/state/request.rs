use crate::error::BubblegumError;
use anchor_lang::prelude::*;

pub const MINT_REQUEST_SIZE: usize = 48 + 8;

#[account]
#[derive(Copy, Debug)]
pub struct MintRequest {
    pub mint_authority: Pubkey,
    pub num_mints_requested: u64,
    pub num_mints_approved: u64,
}

impl MintRequest {
    pub fn init(&mut self, mint_authority: &Pubkey, mint_capacity: u64) {
        self.mint_authority = *mint_authority;
        self.num_mints_requested = mint_capacity;
        self.num_mints_approved = 0;
    }

    pub fn decrement_approvals(&mut self) -> Result<()> {
        if self.num_mints_approved == 0 {
            return Err(BubblegumError::MintRequestNotApproved.into());
        }
        self.num_mints_approved -= 1;
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.mint_authority != Pubkey::default()
    }

    pub fn init_or_set(&mut self, auth: Pubkey, mint_capacity: u64) {
        if self.is_initialized() {
            self.num_mints_requested = mint_capacity;
        } else {
            self.init(&auth, mint_capacity);
        }
    }

    pub fn approve(&mut self, num_to_approve: u64) -> Result<()> {
        if num_to_approve > self.num_mints_requested {
            msg!("Cannot approve more mints than the requested amount");
            return Err(BubblegumError::MintRequestNotApproved.into());
        }
        self.num_mints_requested = self.num_mints_requested.saturating_sub(num_to_approve);
        self.num_mints_approved = self.num_mints_approved.saturating_add(num_to_approve);
        Ok(())
    }

    pub fn process_mint(&mut self) -> Result<()> {
        if self.num_mints_approved > 0 {
            return Err(BubblegumError::MintRequestNotApproved.into());
        }

        self.num_mints_approved = self.num_mints_approved.saturating_sub(1);
        Ok(())
    }

    pub fn has_mint_capacity(&self, capacity: u64) -> bool {
        self.num_mints_approved >= capacity
    }
}
