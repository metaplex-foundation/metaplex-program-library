use super::*;
use crate::instruction::{MetadataDelegateExpiration, MetadataDelegateRole};
use solana_program::sysvar::Sysvar;

const SIZE: usize = 98;

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
/// SEEDS = [
///     "metadata",
///     program id,
///     mint id,
///     delegate role,
///     update authority id,
///     delegate id
/// ]
pub struct MetadataDelegateRecord {
    pub key: Key, // 1
    pub bump: u8, // 1
    #[cfg_attr(feature = "serde-feature", serde(with = "As::<DisplayFromStr>"))]
    pub mint: Pubkey, // 32
    #[cfg_attr(feature = "serde-feature", serde(with = "As::<DisplayFromStr>"))]
    pub delegate: Pubkey, // 32
    #[cfg_attr(feature = "serde-feature", serde(with = "As::<DisplayFromStr>"))]
    pub update_authority: Pubkey, // 32
}

impl Default for MetadataDelegateRecord {
    fn default() -> Self {
        Self {
            key: Key::MetadataDelegate,
            bump: 255,
            mint: Pubkey::default(),
            delegate: Pubkey::default(),
            update_authority: Pubkey::default(),
        }
    }
}

impl TokenMetadataAccount for MetadataDelegateRecord {
    fn key() -> Key {
        Key::MetadataDelegate
    }

    fn size() -> usize {
        SIZE
    }
}

impl MetadataDelegateRecord {
    pub fn from_bytes(data: &[u8]) -> Result<MetadataDelegateRecord, ProgramError> {
        let delegate: MetadataDelegateRecord =
            try_from_slice_checked(data, Key::MetadataDelegate, MetadataDelegateRecord::size())?;
        Ok(delegate)
    }
}

// V2

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
/// SEEDS = [
///     "metadata",
///     program id,
///     mint id,
///     delegate role,
///     update authority id,
///     delegate id
/// ]
pub struct MetadataDelegateRecordV2 {
    pub key: Key, // 1
    pub bump: u8, // 1
    #[cfg_attr(feature = "serde-feature", serde(with = "As::<DisplayFromStr>"))]
    pub mint: Pubkey, // 32
    #[cfg_attr(feature = "serde-feature", serde(with = "As::<DisplayFromStr>"))]
    pub delegate: Pubkey, // 32
    #[cfg_attr(feature = "serde-feature", serde(with = "As::<DisplayFromStr>"))]
    pub update_authority: Pubkey, // 32
    pub creation_time: i64, // 8
}

impl Default for MetadataDelegateRecordV2 {
    fn default() -> Self {
        Self {
            key: Key::MetadataDelegateV2,
            bump: 255,
            mint: Pubkey::default(),
            delegate: Pubkey::default(),
            update_authority: Pubkey::default(),
            creation_time: 0,
        }
    }
}

impl TokenMetadataAccount for MetadataDelegateRecordV2 {
    fn key() -> Key {
        Key::MetadataDelegateV2
    }

    fn size() -> usize {
        SIZE + 8
    }
}

impl MetadataDelegateRecordV2 {
    pub fn from_bytes(data: &[u8]) -> Result<MetadataDelegateRecordV2, ProgramError> {
        let delegate: MetadataDelegateRecordV2 = try_from_slice_checked(
            data,
            Key::MetadataDelegateV2,
            MetadataDelegateRecordV2::size(),
        )?;
        Ok(delegate)
    }

    pub fn check_expiration(&self, role: MetadataDelegateRole) -> ProgramResult {
        // Get the current time.
        let current_time = solana_program::clock::Clock::get()?;

        if current_time.unix_timestamp > self.creation_time + role.expiration_time_secs() {
            Err(MetadataError::DelegateExpired.into())
        } else {
            Ok(())
        }
    }
}
