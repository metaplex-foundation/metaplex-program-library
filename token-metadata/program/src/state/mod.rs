pub(crate) mod collection;
pub(crate) mod creator;
pub(crate) mod data;
pub(crate) mod edition;
pub(crate) mod edition_marker;
pub(crate) mod escrow;
pub(crate) mod master_edition;
pub(crate) mod metadata;
pub(crate) mod reservation;
pub(crate) mod uses;

use std::io::ErrorKind;

use borsh::{maybestd::io::Error as BorshError, BorshDeserialize, BorshSerialize};
pub use collection::*;
pub use creator::*;
pub use data::*;
pub use edition::*;
pub use edition_marker::*;
pub use escrow::*;
pub use master_edition::*;
pub use metadata::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
pub use reservation::*;
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};
pub use uses::*;
#[cfg(feature = "serde-feature")]
use {
    serde::{Deserialize, Serialize},
    serde_with::{As, DisplayFromStr},
};

// Re-export constants to maintain compatibility.
pub use crate::pda::{BURN, COLLECTION_AUTHORITY, EDITION, PREFIX, USER};
use crate::{
    assertions::assert_owned_by,
    error::MetadataError,
    utils::{meta_deser_unchecked, try_from_slice_checked},
    ID,
};

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, FromPrimitive)]
pub enum TokenStandard {
    NonFungible,        // This is a master edition
    FungibleAsset,      // A token with metadata that can also have attrributes
    Fungible,           // A token with simple metadata
    NonFungibleEdition, // This is a limited edition
}

pub trait TokenMetadataAccount: BorshDeserialize {
    fn key() -> Key;

    fn size() -> usize;

    fn is_correct_account_type(data: &[u8], data_type: Key, data_size: usize) -> bool {
        if data.is_empty() {
            return false;
        }

        let key: Option<Key> = Key::from_u8(data[0]);
        match key {
            Some(key) => {
                (key == data_type || key == Key::Uninitialized) && (data.len() == data_size)
            }
            None => false,
        }
    }

    fn pad_length(buf: &mut Vec<u8>) -> Result<(), MetadataError> {
        let padding_length = Self::size()
            .checked_sub(buf.len())
            .ok_or(MetadataError::NumericalOverflowError)?;
        buf.extend(vec![0; padding_length]);
        Ok(())
    }

    fn safe_deserialize(mut data: &[u8]) -> Result<Self, BorshError> {
        if !Self::is_correct_account_type(data, Self::key(), Self::size()) {
            return Err(BorshError::new(ErrorKind::Other, "DataTypeMismatch"));
        }

        let result = Self::deserialize(&mut data)?;

        Ok(result)
    }

    fn from_account_info(a: &AccountInfo) -> Result<Self, ProgramError>
where {
        let ua = Self::safe_deserialize(&a.data.borrow_mut())
            .map_err(|_| MetadataError::DataTypeMismatch)?;

        // Check that this is a `token-metadata` owned account.
        assert_owned_by(a, &ID)?;

        Ok(ua)
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Copy, FromPrimitive)]
pub enum Key {
    Uninitialized,
    EditionV1,
    MasterEditionV1,
    ReservationListV1,
    MetadataV1,
    ReservationListV2,
    MasterEditionV2,
    EditionMarker,
    UseAuthorityRecord,
    CollectionAuthorityRecord,
    TokenOwnedEscrow,
}
