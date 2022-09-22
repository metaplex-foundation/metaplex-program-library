use crate::state::{EscrowAuthority, ESCROW_PREFIX, PREFIX};
use borsh::BorshSerialize;
use solana_program::pubkey::Pubkey;

pub fn find_escrow_account(mint: &Pubkey, authority: &EscrowAuthority) -> (Pubkey, u8) {
    let authority_primitive = authority.try_to_vec().unwrap();
    Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            crate::id().as_ref(),
            mint.as_ref(),
            authority_primitive.as_ref(),
            ESCROW_PREFIX.as_ref(),
        ],
        &crate::id(),
    )
}
