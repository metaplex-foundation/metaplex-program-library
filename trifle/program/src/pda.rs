use crate::state::TRIFLE_SEED;
use solana_program::pubkey::Pubkey;
/// Trifle account PDA seeds
///     "trifle",
///     mint.key.as_ref(),
///     trifle_authority_info.key.as_ref(),
///     escrow_constraint_model.key.as_ref(),
pub fn find_trifle_address(
    mint: &Pubkey,
    trifle_authority: &Pubkey,
    escrow_constraint_model: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    let seeds = [
        TRIFLE_SEED.as_bytes(),
        mint.as_ref(),
        trifle_authority.as_ref(),
        escrow_constraint_model.as_ref(),
    ];
    Pubkey::find_program_address(&seeds, program_id)
}
