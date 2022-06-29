use solana_program_test::*;

pub fn setup_program<'a>() -> ProgramTest {
    let mut program = ProgramTest::new("mpl_listing_rewards", mpl_listing_rewards::id(), None);
    program.add_program("mpl_auction_house", mpl_auction_house::id(), None);
    program.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);

    program
}
