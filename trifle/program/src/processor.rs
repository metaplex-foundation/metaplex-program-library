use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub fn process_instruction(
    program_id: Pubkey,
    accounts: &[AccountInfo],
    args: &[u8],
) -> ProgramResult {
    Ok(())
}
