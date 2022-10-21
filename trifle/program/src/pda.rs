use crate::state::{ESCROW_SEED, FREEZE_AUTHORITY, TRIFLE_SEED};
use solana_program::pubkey::Pubkey;

/// Trifle account PDA seeds
///     "trifle",
///     mint.key.as_ref(),
///     trifle_authority_info.key.as_ref(),
pub fn find_trifle_address(mint: &Pubkey, trifle_authority: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            TRIFLE_SEED.as_bytes(),
            mint.as_ref(),
            trifle_authority.as_ref(),
        ],
        &crate::id(),
    )
}

/// Escrow constraint model PDA seeds
///    "escrow",
///    creator.key.as_ref(),
///    name.as_bytes(),
pub fn find_escrow_constraint_model_address(creator: &Pubkey, name: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[ESCROW_SEED.as_bytes(), creator.as_ref(), name.as_bytes()],
        &crate::id(),
    )
}

/// Program Freeze Authority PDA seeds
/// "escrow"
/// "freeze_authority"
pub fn find_program_freeze_authority_address(escrow_constraint_model: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            ESCROW_SEED.as_bytes(),
            escrow_constraint_model.as_ref(),
            FREEZE_AUTHORITY.as_bytes(),
        ],
        &crate::id(),
    )
}
