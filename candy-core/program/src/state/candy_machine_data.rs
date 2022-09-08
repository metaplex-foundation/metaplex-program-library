use anchor_lang::prelude::*;
use mpl_token_metadata::state::{MAX_CREATOR_LIMIT, MAX_NAME_LENGTH, MAX_URI_LENGTH};

use crate::{constants::HIDDEN_SECTION, errors::CandyError, utils::replace_patterns};

/// Candy machine configuration data.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Debug)]
pub struct CandyMachineData {
    /// Number of assets available
    pub items_available: u64,
    /// Symbol for the asset
    pub symbol: String,
    /// Secondary sales royalty basis points (0-10000)
    pub seller_fee_basis_points: u16,
    /// Max supply of each individual asset (default 0)
    pub max_supply: u64,
    /// Indicates if the asset is mutable or not (default yes)
    pub is_mutable: bool,
    /// List of creators
    pub creators: Vec<Creator>,
    /// Config line settings
    pub config_line_settings: Option<ConfigLineSettings>,
    /// Hidden setttings
    pub hidden_settings: Option<HiddenSettings>,
}

// Creator information.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Creator {
    /// Pubkey address
    pub address: Pubkey,
    /// Whether the creator is verified or not
    pub verified: bool,
    // Share of secondary sales royalty
    pub percentage_share: u8,
}

/// Hidden settings for large mints used with off-chain data.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Debug)]
pub struct HiddenSettings {
    /// Asset prefix name
    pub name: String,
    /// Shared URI
    pub uri: String,
    /// Hash of the hidden settings file
    pub hash: [u8; 32],
}

/// Config line settings to allocate space for individual name + URI.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Debug)]
pub struct ConfigLineSettings {
    /// Common name prefix
    pub prefix_name: String,
    /// Length of the remaining part of the name
    pub name_length: u32,
    /// Common URI prefix
    pub prefix_uri: String,
    /// Length of the remaining part of the URI
    pub uri_length: u32,
    /// Indicates whether to use a senquential index generator or not
    pub is_sequential: bool,
}

impl CandyMachineData {
    pub fn get_space_for_candy(&self) -> Result<usize> {
        Ok(if self.hidden_settings.is_some() {
            HIDDEN_SECTION
        } else {
            HIDDEN_SECTION
                + 4
                + (self.items_available as usize) * self.get_config_line_size()
                + 4
                + ((self
                    .items_available
                    .checked_div(8)
                    .ok_or(CandyError::NumericalOverflowError)?
                    + 1) as usize)
                + 4
                + (self.items_available as usize) * 4
        })
    }

    pub fn get_config_line_size(&self) -> usize {
        if let Some(config_line) = &self.config_line_settings {
            (config_line.name_length + config_line.uri_length) as usize
        } else {
            0
        }
    }

    /// Validates the hidden and config lines settings against the maximum
    /// allowed values for name and URI.
    ///
    /// Hidden settings take precedence over config lines since when hidden
    /// settings are used, the account does not need to include space for
    /// config lines.
    pub fn validate(&self) -> Result<()> {
        // validation substitutes any variable for the maximum allowed index
        // to check the longest possible name and uri that can result from the
        // replacement of the variables

        if let Some(hidden) = &self.hidden_settings {
            let expected = replace_patterns(hidden.name.clone(), self.items_available as usize);
            if MAX_NAME_LENGTH < expected.len() {
                return err!(CandyError::ExceededLengthError);
            }

            let expected = replace_patterns(hidden.uri.clone(), self.items_available as usize);
            if MAX_URI_LENGTH < expected.len() {
                return err!(CandyError::ExceededLengthError);
            }
        } else if let Some(config_line) = &self.config_line_settings {
            let expected = replace_patterns(
                config_line.prefix_name.clone(),
                self.items_available as usize,
            );
            if MAX_NAME_LENGTH < (expected.len() + config_line.name_length as usize) {
                return err!(CandyError::ExceededLengthError);
            }

            let expected = replace_patterns(
                config_line.prefix_uri.clone(),
                self.items_available as usize,
            );
            if MAX_URI_LENGTH < (expected.len() + config_line.uri_length as usize) {
                return err!(CandyError::ExceededLengthError);
            }
        } else {
            return err!(CandyError::MissingConfigLinesSettings);
        }

        // (MAX_CREATOR_LIMIT - 1) because the candy machine is going to be a creator
        if self.creators.len() > (MAX_CREATOR_LIMIT - 1) {
            return err!(CandyError::TooManyCreators);
        }

        Ok(())
    }
}
