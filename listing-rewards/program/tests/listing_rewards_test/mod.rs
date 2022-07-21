pub mod fixtures;

use solana_program_test::*;

pub const TEN_SOL: u64 = 10_000_000_000;
pub const ONE_SOL: u64 = 1_000_000_000;
pub const TEST_COLLECTION: &str = "Cehzo7ugAvuYcTst9HF24ackLxnrpDkzHFajj17FuyUR";

pub fn setup_program<'a>() -> ProgramTest {
    let mut program = ProgramTest::new("mpl_listing_rewards", mpl_listing_rewards::id(), None);
    program.add_program("mpl_auction_house", mpl_auction_house::id(), None);
    program.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);

    program
}
