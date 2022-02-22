use crate::{CreateSecondaryMetadataCreators, utils::*, error::ErrorCode,};
use anchor_lang::prelude::*;

impl<'info> CreateSecondaryMetadataCreators<'info> {
    pub fn process(
        &mut self,
        _secondary_metadata_creators_bump: u8,
        creators: Vec<mpl_token_metadata::state::Creator>,
    ) -> ProgramResult {
        let metadata = &self.metadata;
        let admin = &self.admin;
        let secondary_metadata_creators = &mut self.secondary_metadata_creators;
        let metadata_state = mpl_token_metadata::state::Metadata::from_account_info(&metadata)?;

        if creators.len() > MAX_SECONDARY_CREATORS_LEN {
            return Err(ErrorCode::CreatorsIsGtThanAvailable.into());
        }

        if creators.is_empty() {
            return Err(ErrorCode::CreatorsIsEmpty.into());
        }

        if !metadata_state.is_mutable {
            return Err(ErrorCode::MetadataShouldBeMutable.into());
        }

        if metadata_state.primary_sale_happened {
            return Err(ErrorCode::PrimarySaleIsNotAllowed.into());
        }

        assert_keys_equal(metadata_state.update_authority, *admin.key)?;

        secondary_metadata_creators.creators = creators;

        Ok(())
    }
}