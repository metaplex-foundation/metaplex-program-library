mod create_medatata_accounts_v2;
mod create_medatata_accounts_v3;
mod puff_metadata;
mod remove_creator_verification;
mod set_token_standard;
mod sign_metadata;
mod update_metadata_account_v2;
mod update_primary_sale_happened_via_token;

pub use create_medatata_accounts_v2::process_create_metadata_accounts_v2;
pub use create_medatata_accounts_v3::process_create_metadata_accounts_v3;
pub use puff_metadata::process_puff_metadata_account;
pub use remove_creator_verification::process_remove_creator_verification;
pub use set_token_standard::process_set_token_standard;
pub use sign_metadata::process_sign_metadata;
pub use update_metadata_account_v2::process_update_metadata_accounts_v2;
pub use update_primary_sale_happened_via_token::process_update_primary_sale_happened_via_token;

pub(crate) mod metadata_instructions {
    pub use create_medatata_accounts_v2::instruction::*;
    pub use create_medatata_accounts_v3::instruction::*;
    pub use puff_metadata::instruction::*;
    pub use remove_creator_verification::instruction::*;
    pub use set_token_standard::instruction::*;
    pub use sign_metadata::instruction::*;
    pub use update_metadata_account_v2::instruction::*;
    pub use update_primary_sale_happened_via_token::instruction::*;

    use super::*;
}
