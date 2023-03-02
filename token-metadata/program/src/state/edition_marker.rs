use super::*;

pub const MAX_EDITION_MARKER_SIZE: usize = 32;

pub const EDITION_MARKER_BIT_SIZE: u64 = 248;

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
pub struct EditionMarker {
    pub key: Key,
    pub ledger: [u8; 31],
}

impl Default for EditionMarker {
    fn default() -> Self {
        Self {
            key: Key::EditionMarker,
            ledger: [0; 31],
        }
    }
}

impl TokenMetadataAccount for EditionMarker {
    fn key() -> Key {
        Key::EditionMarker
    }

    fn size() -> usize {
        MAX_EDITION_MARKER_SIZE
    }
}

impl EditionMarker {
    fn get_edition_offset_from_starting_index(edition: u64) -> Result<usize, ProgramError> {
        Ok(edition
            .checked_rem(EDITION_MARKER_BIT_SIZE)
            .ok_or(MetadataError::NumericalOverflowError)? as usize)
    }

    fn get_index(offset_from_start: usize) -> Result<usize, ProgramError> {
        let index = offset_from_start
            .checked_div(8)
            .ok_or(MetadataError::NumericalOverflowError)?;

        // With only EDITION_MARKER_BIT_SIZE bits, or 31 bytes, we have a max constraint here.
        if index > 30 {
            return Err(MetadataError::InvalidEditionIndex.into());
        }

        Ok(index)
    }

    fn get_offset_from_right(offset_from_start: usize) -> Result<u32, ProgramError> {
        // We're saying the left hand side of a u8 is the 0th index so to get a 1 in that 0th index
        // you need to shift a 1 over 8 spots from the right hand side. To do that you actually
        // need not 00000001 but 10000000 which you can get by simply multiplying 1 by 2^7, 128 and then ORing
        // it with the current value.
        Ok(7 - offset_from_start
            .checked_rem(8)
            .ok_or(MetadataError::NumericalOverflowError)? as u32)
    }

    pub fn get_index_and_mask(edition: u64) -> Result<(usize, u8), ProgramError> {
        // How many editions off we are from edition at 0th index
        let offset_from_start = EditionMarker::get_edition_offset_from_starting_index(edition)?;

        // How many whole u8s we are from the u8 at the 0th index, which basically dividing by 8
        let index = EditionMarker::get_index(offset_from_start)?;

        // what position in the given u8 bitset are we (remainder math)
        let my_position_in_index_starting_from_right =
            EditionMarker::get_offset_from_right(offset_from_start)?;

        Ok((index, u8::pow(2, my_position_in_index_starting_from_right)))
    }

    pub fn edition_taken(&self, edition: u64) -> Result<bool, ProgramError> {
        let (index, mask) = EditionMarker::get_index_and_mask(edition)?;

        // apply mask with bitwise and with a 1 to determine if it is set or not
        let applied_mask = self.ledger[index] & mask;

        // What remains should not equal 0.
        Ok(applied_mask != 0)
    }

    pub fn insert_edition(&mut self, edition: u64) -> ProgramResult {
        let (index, mask) = EditionMarker::get_index_and_mask(edition)?;
        // bitwise or a 1 into our position in that position
        self.ledger[index] |= mask;
        Ok(())
    }

    pub fn save(self, account_info: &AccountInfo) -> ProgramResult {
        // Clear all data to ensure it is serialized cleanly.
        let mut edition_marker_data = account_info.try_borrow_mut_data()?;
        edition_marker_data[0..].fill(0);

        borsh::BorshSerialize::serialize(&self, &mut *edition_marker_data)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use borsh::BorshSerialize;
    use solana_program::account_info::AccountInfo;
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::{
        error::MetadataError,
        state::{EditionMarker, Key, Metadata, TokenMetadataAccount},
        ID,
    };

    #[test]
    fn successfully_deserialize() {
        let expected_data = EditionMarker::default();

        let mut buf = Vec::new();
        expected_data.serialize(&mut buf).unwrap();
        EditionMarker::pad_length(&mut buf).unwrap();

        let pubkey = Keypair::new().pubkey();
        let owner = &ID;
        let mut lamports = 1_000_000_000;
        let mut data = buf.clone();

        let account_info = AccountInfo::new(
            &pubkey,
            false,
            true,
            &mut lamports,
            &mut data,
            owner,
            false,
            1_000_000_000,
        );

        let data = EditionMarker::from_account_info(&account_info).unwrap();
        assert_eq!(data.key, Key::EditionMarker);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn deserializing_wrong_account_type_fails() {
        let wrong_type = Metadata::default();

        let mut buf = Vec::new();
        wrong_type.serialize(&mut buf).unwrap();
        Metadata::pad_length(&mut buf).unwrap();

        let pubkey = Keypair::new().pubkey();
        let owner = &ID;
        let mut lamports = 1_000_000_000;
        let mut data = buf.clone();

        let account_info = AccountInfo::new(
            &pubkey,
            false,
            true,
            &mut lamports,
            &mut data,
            owner,
            false,
            1_000_000_000,
        );

        let error = EditionMarker::from_account_info(&account_info).unwrap_err();
        assert_eq!(error, MetadataError::DataTypeMismatch.into());
    }
}
