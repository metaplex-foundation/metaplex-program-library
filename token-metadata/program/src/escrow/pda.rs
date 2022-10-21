use crate::state::{EscrowAuthority, ESCROW_PREFIX, PREFIX};
use solana_program::pubkey::Pubkey;

pub fn find_escrow_account(mint: &Pubkey, authority: &EscrowAuthority) -> (Pubkey, u8) {
    let id = crate::id();
    let mut seeds = vec![PREFIX.as_bytes(), id.as_ref(), mint.as_ref()];

    for seed in authority.to_seeds() {
        seeds.push(seed);
    }

    seeds.push(ESCROW_PREFIX.as_bytes());

    Pubkey::find_program_address(&seeds, &crate::id())
}
