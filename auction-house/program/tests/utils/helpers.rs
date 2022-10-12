use std::env;

use anchor_lang::prelude::Pubkey;
use mpl_auction_house::{
    constants::{FEE_PAYER, MAX_NUM_SCOPES, PREFIX, TREASURY},
    AuthorityScope,
};
use mpl_testing_utils::assert_error;
use solana_program::instruction::InstructionError;
use solana_program_test::BanksClientError;
use solana_sdk::transaction::TransactionError;

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

/// In CI we're running into IoError(the request exceeded its deadline) which is most likely a
/// timing issue that happens due to decreased performance.
/// Increasing compute limits seems to have made this happen less often, but for a few tests we
/// still observe this behavior which makes tests fail in CI for the wrong reason.
/// The below is a workaround to make it even less likely.
/// Tests are still brittle, but fail much less often which is the best we can do for now aside
/// from disabling the problematic tests in CI entirely.
pub fn assert_error_ignoring_io_error_in_ci(error: &BanksClientError, error_code: u32) {
    match error {
        BanksClientError::Io(err) if env::var("CI").is_ok() => {
            match err.kind() {
                std::io::ErrorKind::Other
                    if &err.to_string() == "the request exceeded its deadline" =>
                {
                    eprintln!("Encountered {:#?} error", err);
                    eprintln!("However since we are running in CI this is acceptable and we can ignore it");
                }
                _ => {
                    eprintln!("Encountered {:#?} error ({})", err, err);
                    panic!("Encountered unknown IoError");
                }
            }
        }
        _ => {
            assert_error!(error, &error_code)
        }
    }
}

/// See `assert_error_ignoring_io_error_in_ci` for more details regarding this workaround
pub fn unwrap_ignoring_io_error_in_ci(result: Result<(), BanksClientError>) {
    match result {
        Ok(()) => (),
        Err(error) => match error {
            BanksClientError::Io(err) if env::var("CI").is_ok() => match err.kind() {
                std::io::ErrorKind::Other
                    if &err.to_string() == "the request exceeded its deadline" =>
                {
                    eprintln!("Encountered {:#?} error", err);
                    eprintln!("However since we are running in CI this is acceptable and we can ignore it");
                }
                _ => {
                    eprintln!("Encountered {:#?} error ({})", err, err);
                    panic!("Encountered unknown IoError");
                }
            },
            _ => {
                panic!("Encountered: {:#?}", error);
            }
        },
    }
}
