mod create_master_edition;
mod deprecated_create_metadata_accounts;
mod deprecated_update_metadata_accounts;
mod mint_new_edition_from_master_edition_via_vault_proxy;

pub use deprecated_create_metadata_accounts::process_deprecated_create_metadata_accounts;
pub use deprecated_update_metadata_accounts::process_deprecated_update_metadata_accounts;
pub use mint_new_edition_from_master_edition_via_vault_proxy::process_deprecated_mint_new_edition_from_master_edition_via_vault_proxy;

pub(crate) mod deprecated_instructions {
    pub use args::*;
    pub use create_master_edition::instruction::*;
    pub use deprecated_create_metadata_accounts::instruction::*;
    pub use deprecated_update_metadata_accounts::instruction::*;
    pub use mint_new_edition_from_master_edition_via_vault_proxy::instruction::*;

    use super::*;
}

mod args {
    use borsh::{BorshDeserialize, BorshSerialize};
    #[cfg(feature = "serde-feature")]
    use serde::{Deserialize, Serialize};

    use crate::state::Reservation;

    #[repr(C)]
    #[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
    #[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
    pub struct MintPrintingTokensViaTokenArgs {
        pub supply: u64,
    }

    #[repr(C)]
    #[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
    #[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
    pub struct SetReservationListArgs {
        /// If set, means that no more than this number of editions can ever be minted. This is immutable.
        pub reservations: Vec<Reservation>,
        /// should only be present on the very first call to set reservation list.
        pub total_reservation_spots: Option<u64>,
        /// Where in the reservation list you want to insert this slice of reservations
        pub offset: u64,
        /// What the total spot offset is in the reservation list from the beginning to your slice of reservations.
        /// So if is going to be 4 total editions eventually reserved between your slice and the beginning of the array,
        /// split between 2 reservation entries, the offset variable above would be "2" since you start at entry 2 in 0 indexed array
        /// (first 2 taking 0 and 1) and because they each have 2 spots taken, this variable would be 4.
        pub total_spot_offset: u64,
    }
}
