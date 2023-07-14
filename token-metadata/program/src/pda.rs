use solana_program::pubkey::Pubkey;

use crate::{instruction::MetadataDelegateRole, state::TOKEN_RECORD_SEED};

/// prefix used for PDAs to avoid certain collision attacks:
/// <https://en.wikipedia.org/wiki/Collision_attack#Chosen-prefix_collision_attack>

pub const PREFIX: &str = "metadata";

pub const EDITION: &str = "edition";

pub const MARKER: &str = "marker";

pub const USER: &str = "user";

pub const BURN: &str = "burn";

pub const COLLECTION_AUTHORITY: &str = "collection_authority";

pub fn find_edition_account(mint: &Pubkey, edition_number: String) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            crate::ID.as_ref(),
            mint.as_ref(),
            EDITION.as_bytes(),
            edition_number.as_bytes(),
        ],
        &crate::ID,
    )
}

pub fn find_master_edition_account(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            crate::ID.as_ref(),
            mint.as_ref(),
            EDITION.as_bytes(),
        ],
        &crate::ID,
    )
}

pub fn find_metadata_account(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[PREFIX.as_bytes(), crate::ID.as_ref(), mint.as_ref()],
        &crate::ID,
    )
}

pub fn find_use_authority_account(mint: &Pubkey, authority: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            crate::ID.as_ref(),
            mint.as_ref(),
            USER.as_bytes(),
            authority.as_ref(),
        ],
        &crate::ID,
    )
}

pub fn find_collection_authority_account(mint: &Pubkey, authority: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            crate::ID.as_ref(),
            mint.as_ref(),
            COLLECTION_AUTHORITY.as_bytes(),
            authority.as_ref(),
        ],
        &crate::ID,
    )
}

pub fn find_program_as_burner_account() -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[PREFIX.as_bytes(), crate::ID.as_ref(), BURN.as_bytes()],
        &crate::ID,
    )
}

pub fn find_metadata_delegate_record_account(
    mint: &Pubkey,
    role: MetadataDelegateRole,
    update_authority: &Pubkey,
    delegate: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            crate::ID.as_ref(),
            mint.as_ref(),
            role.to_string().as_bytes(),
            update_authority.as_ref(),
            delegate.as_ref(),
        ],
        &crate::ID,
    )
}

pub fn find_token_record_account(mint: &Pubkey, token: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            crate::ID.as_ref(),
            mint.as_ref(),
            TOKEN_RECORD_SEED.as_bytes(),
            token.as_ref(),
        ],
        &crate::ID,
    )
}
