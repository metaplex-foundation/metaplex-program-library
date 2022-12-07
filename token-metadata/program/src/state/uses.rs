use super::*;

pub const USE_AUTHORITY_RECORD_SIZE: usize = 18; //8 byte padding

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, FromPrimitive)]
pub enum UseMethod {
    Burn,
    Multiple,
    Single,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct Uses {
    // 17 bytes + Option byte
    pub use_method: UseMethod, //1
    pub remaining: u64,        //8
    pub total: u64,            //8
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
pub struct UseAuthorityRecord {
    pub key: Key,          //1
    pub allowed_uses: u64, //8
    pub bump: u8,
}

impl Default for UseAuthorityRecord {
    fn default() -> Self {
        UseAuthorityRecord {
            key: Key::UseAuthorityRecord,
            allowed_uses: 0,
            bump: 255,
        }
    }
}

impl TokenMetadataAccount for UseAuthorityRecord {
    fn key() -> Key {
        Key::UseAuthorityRecord
    }

    fn size() -> usize {
        USE_AUTHORITY_RECORD_SIZE
    }
}

impl UseAuthorityRecord {
    pub fn from_bytes(b: &[u8]) -> Result<UseAuthorityRecord, ProgramError> {
        let ua: UseAuthorityRecord =
            try_from_slice_checked(b, Key::UseAuthorityRecord, USE_AUTHORITY_RECORD_SIZE)?;
        Ok(ua)
    }

    pub fn bump_empty(&self) -> bool {
        self.bump == 0 && self.key == Key::UseAuthorityRecord
    }
}

#[cfg(test)]
mod tests {
    use borsh::BorshSerialize;
    use solana_program::account_info::AccountInfo;
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::{
        error::MetadataError,
        state::{CollectionAuthorityRecord, Key, TokenMetadataAccount, UseAuthorityRecord},
        ID,
    };

    #[test]
    fn successfully_deserialize() {
        let expected_data = UseAuthorityRecord::default();

        let mut buf = Vec::new();
        expected_data.serialize(&mut buf).unwrap();
        UseAuthorityRecord::pad_length(&mut buf).unwrap();

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

        let data = UseAuthorityRecord::from_account_info(&account_info).unwrap();
        assert_eq!(data.key, Key::UseAuthorityRecord);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn deserializing_wrong_account_type_fails() {
        let wrong_type = CollectionAuthorityRecord::default();

        let mut buf = Vec::new();
        wrong_type.serialize(&mut buf).unwrap();

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

        let error = UseAuthorityRecord::from_account_info(&account_info).unwrap_err();
        assert_eq!(error, MetadataError::DataTypeMismatch.into());
    }
}
