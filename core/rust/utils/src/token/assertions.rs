use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token_2022::extension::{BaseState, StateWithExtensions};

pub static TOKEN_PROGRAM_IDS: [&Pubkey; 2] = [&spl_token::ID, &spl_token_2022::ID];

pub fn assert_token_program_matches_package(
    token_program_info: &AccountInfo,
    error: impl Into<ProgramError>,
) -> ProgramResult {
    if !TOKEN_PROGRAM_IDS.contains(&token_program_info.key) {
        Err(error.into())
    } else {
        Ok(())
    }
}

pub fn unpack_with_error<S: BaseState>(
    account_data: &[u8],
    error: impl Into<ProgramError>,
) -> Result<StateWithExtensions<'_, S>, ProgramError> {
    StateWithExtensions::<S>::unpack(account_data).map_err(|_| error.into())
}
