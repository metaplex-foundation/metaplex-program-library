use super::*;

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
