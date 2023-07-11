use solana_program::program_memory::sol_memcpy;

use super::*;

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
pub struct EditionMarkerV2 {
    pub key: Key,
    pub ledger: Vec<u8>,
}

impl Default for EditionMarkerV2 {
    fn default() -> Self {
        Self {
            key: Key::EditionMarkerV2,
            ledger: vec![],
        }
    }
}

impl TokenMetadataAccount for EditionMarkerV2 {
    fn key() -> Key {
        Key::EditionMarkerV2
    }

    fn size() -> usize {
        0
    }
}

impl EditionMarkerV2 {
    fn get_index(offset_from_start: usize) -> Result<usize, ProgramError> {
        let index = offset_from_start
            .checked_div(8)
            .ok_or(MetadataError::NumericalOverflowError)?;

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
        let edition = edition
            .try_into()
            .map_err(|_| MetadataError::NumericalOverflowError)?;
        // How many whole u8s we are from the u8 at the 0th index, which basically dividing by 8
        let index = EditionMarkerV2::get_index(edition)?;

        // what position in the given u8 bitset are we (remainder math)
        let my_position_in_index_starting_from_right =
            EditionMarkerV2::get_offset_from_right(edition)?;

        Ok((index, 1u8 << my_position_in_index_starting_from_right))
    }

    pub fn edition_taken(&self, edition: u64) -> Result<bool, ProgramError> {
        let (index, mask) = EditionMarkerV2::get_index_and_mask(edition)?;

        // If the ledger is smaller than the index, then it's not taken.
        if self.ledger.len() <= index {
            Ok(false)
        } else {
            // apply mask with bitwise and with a 1 to determine if it is set or not
            let applied_mask = self.ledger[index] & mask;

            // What remains should not equal 0.
            Ok(applied_mask != 0)
        }
    }

    pub fn insert_edition(&mut self, edition: u64) -> ProgramResult {
        let (index, mask) = EditionMarkerV2::get_index_and_mask(edition)?;

        // If the ledger is smaller than the index, then we need to resize it.
        if self.ledger.len() <= index {
            self.ledger.resize(index + 1, 0);
        }

        // bitwise or a 1 into our position in that position
        self.ledger[index] |= mask;
        Ok(())
    }

    pub fn save<'a>(
        self,
        account_info: &AccountInfo<'a>,
        payer_info: &AccountInfo<'a>,
        system_info: &AccountInfo<'a>,
    ) -> ProgramResult {
        let serialized_data = self
            .try_to_vec()
            .map_err(|_| MetadataError::BorshSerializationError)?;

        resize_or_reallocate_account_raw(
            account_info,
            payer_info,
            system_info,
            serialized_data.len(),
        )?;

        sol_memcpy(
            &mut account_info.try_borrow_mut_data()?,
            &serialized_data,
            serialized_data.len(),
        );

        Ok(())
    }
}
