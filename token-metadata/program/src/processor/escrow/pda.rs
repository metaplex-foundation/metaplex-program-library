use solana_program::pubkey::Pubkey;

use crate::state::{EscrowAuthority, ESCROW_POSTFIX, PREFIX};

pub fn find_escrow_seeds<'a>(mint: &'a Pubkey, authority: &'a EscrowAuthority) -> Vec<&'a [u8]> {
    let mut seeds = vec![PREFIX.as_bytes(), crate::ID.as_ref(), mint.as_ref()];

    for seed in authority.to_seeds() {
        seeds.push(seed);
    }

    seeds.push(ESCROW_POSTFIX.as_bytes());

    seeds
}

pub fn find_escrow_account(mint: &Pubkey, authority: &EscrowAuthority) -> (Pubkey, u8) {
    let seeds = find_escrow_seeds(mint, authority);
    Pubkey::find_program_address(&seeds, &crate::ID)
}
