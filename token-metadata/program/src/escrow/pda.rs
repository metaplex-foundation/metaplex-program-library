use crate::state::{ESCROW_PREFIX, PREFIX};
use solana_program::pubkey::Pubkey;

pub fn find_escrow_account(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            crate::id().as_ref(),
            mint.as_ref(),
            ESCROW_PREFIX.as_ref(),
        ],
        &crate::id(),
    )
}

pub fn find_escrow_constraint_model_account(creator: &Pubkey, name: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            crate::id().as_ref(),
            ESCROW_PREFIX.as_ref(),
            creator.as_ref(),
            name.as_bytes(),
        ],
        &crate::id(),
    )
}
