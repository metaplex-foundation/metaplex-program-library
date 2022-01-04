pub mod helpers;

use solana_program_test::*;

pub fn membership_token_program_test() -> ProgramTest {
    let mut program_test =
        ProgramTest::new("mpl_membership_token", mpl_membership_token::id(), None);
    program_test.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);

    program_test
}
