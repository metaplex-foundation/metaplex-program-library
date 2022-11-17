mod convert_master_edition_v1_to_v2;
mod create_master_edition_v3;
mod mint_new_edition_from_master_edition_via_token;

pub use convert_master_edition_v1_to_v2::process_convert_master_edition_v1_to_v2;
pub use create_master_edition_v3::process_create_master_edition;
pub use mint_new_edition_from_master_edition_via_token::process_mint_new_edition_from_master_edition_via_token;

pub(crate) mod edition_instructions {
    pub use convert_master_edition_v1_to_v2::instruction::*;
    pub use create_master_edition_v3::instruction::*;
    pub use mint_new_edition_from_master_edition_via_token::instruction::*;

    use super::*;
}
