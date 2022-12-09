use crate::instruction::DelegateRole;

use super::*;

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
pub struct Delegate {
    pub key: Key,           // 1
    pub role: DelegateRole, // 1
    pub bump: u8,           // 1
}

impl Default for Delegate {
    fn default() -> Self {
        Self {
            key: Key::Delegate,
            role: DelegateRole::Authority,
            bump: 255,
        }
    }
}

impl TokenMetadataAccount for Delegate {
    fn key() -> Key {
        Key::CollectionAuthorityRecord
    }

    fn size() -> usize {
        std::mem::size_of::<Delegate>()
    }
}

impl Delegate {
    pub fn from_bytes(data: &[u8]) -> Result<Delegate, ProgramError> {
        let delegate: Delegate = try_from_slice_checked(data, Key::Delegate, Delegate::size())?;
        Ok(delegate)
    }
}
