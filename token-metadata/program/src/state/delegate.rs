use super::*;

const SIZE: usize = 2;

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
pub struct MetadataDelegateRecord {
    pub key: Key, // 1
    pub bump: u8, // 1
}

impl Default for MetadataDelegateRecord {
    fn default() -> Self {
        Self {
            key: Key::MetadataDelegate,
            bump: 255,
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
