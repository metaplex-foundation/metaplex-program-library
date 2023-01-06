use super::*;
use crate::instruction::DelegateRole;

pub const PERSISTENT_DELEGATE: &str = "persistent_delegate";

const SIZE: usize = 35;

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
pub struct DelegateRecord {
    pub key: Key,           // 1
    pub bump: u8,           // 1
    pub role: DelegateRole, // 1
    pub delegate: Pubkey,   // 32
}

impl Default for DelegateRecord {
    fn default() -> Self {
        Self {
            key: Key::Delegate,
            role: DelegateRole::Authority,
            bump: 255,
            delegate: Pubkey::default(),
        }
    }
}

impl TokenMetadataAccount for DelegateRecord {
    fn key() -> Key {
        Key::Delegate
    }

    fn size() -> usize {
        SIZE
    }
}

impl DelegateRecord {
    pub fn from_bytes(data: &[u8]) -> Result<DelegateRecord, ProgramError> {
        let delegate: DelegateRecord =
            try_from_slice_checked(data, Key::Delegate, DelegateRecord::size())?;
        Ok(delegate)
    }
}
