use anchor_lang::prelude::Pubkey;
use mpl_auction_house::{
    constants::{FEE_PAYER, MAX_NUM_SCOPES, PREFIX, TREASURY},
    AuthorityScope,
};

pub fn default_scopes() -> Vec<AuthorityScope> {
    vec![
        AuthorityScope::Deposit,
        AuthorityScope::Buy,
        AuthorityScope::PublicBuy,
        AuthorityScope::ExecuteSale,
        AuthorityScope::Sell,
        AuthorityScope::Cancel,
        AuthorityScope::Withdraw,
    ]
}

pub fn assert_scopes_eq(scopes: Vec<AuthorityScope>, scopes_array: [bool; MAX_NUM_SCOPES]) {
    for scope in scopes {
        if !scopes_array[scope as usize] {
            panic!();
        }
    }
}

/// Find a valid program address and its corresponding bump seed which must be passed
/// as an additional seed when calling `invoke_signed`
#[allow(clippy::same_item_push)]
pub fn find_noncanonical_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
    let mut bump_seed = [std::u8::MAX];
    let mut found_canonical = false;
    for _ in 0..std::u8::MAX {
        {
            let mut seeds_with_bump = seeds.to_vec();
            seeds_with_bump.push(&bump_seed);
            if let Ok(address) = Pubkey::create_program_address(&seeds_with_bump, program_id) {
                if found_canonical {
                    return (address, bump_seed[0]);
                } else {
                    found_canonical = true;
                }
            }
        }
        bump_seed[0] -= 1;
    }
    panic!("Unable to find a viable program address bump seed");
}

pub fn find_noncanonical_auction_house_fee_account_address(
    auction_house_address: &Pubkey,
) -> (Pubkey, u8) {
    let auction_fee_account_seeds = &[
        PREFIX.as_bytes(),
        auction_house_address.as_ref(),
        FEE_PAYER.as_bytes(),
    ];
    find_noncanonical_program_address(auction_fee_account_seeds, &mpl_auction_house::id())
}

pub fn find_noncanonical_auction_house_treasury_address(
    auction_house_address: &Pubkey,
) -> (Pubkey, u8) {
    let auction_house_treasury_seeds = &[
        PREFIX.as_bytes(),
        auction_house_address.as_ref(),
        TREASURY.as_bytes(),
    ];
    find_noncanonical_program_address(auction_house_treasury_seeds, &mpl_auction_house::id())
}
