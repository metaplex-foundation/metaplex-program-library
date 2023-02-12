use super::*;

pub const COLLECTION_AUTHORITY_RECORD_SIZE: usize = 35;

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct Collection {
    pub verified: bool,
    #[cfg_attr(feature = "serde-feature", serde(with = "As::<DisplayFromStr>"))]
    pub key: Pubkey,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
pub struct CollectionAuthorityRecord {
    pub key: Key,                         //1
    pub bump: u8,                         //1
    pub update_authority: Option<Pubkey>, //33 (1 + 32)
}

impl Default for CollectionAuthorityRecord {
    fn default() -> Self {
        CollectionAuthorityRecord {
            key: Key::CollectionAuthorityRecord,
            bump: 255,
            update_authority: None,
        }
    }
}

impl TokenMetadataAccount for CollectionAuthorityRecord {
    fn key() -> Key {
        Key::CollectionAuthorityRecord
    }

    fn size() -> usize {
        COLLECTION_AUTHORITY_RECORD_SIZE
    }
}

impl CollectionAuthorityRecord {
    pub fn from_bytes(b: &[u8]) -> Result<CollectionAuthorityRecord, ProgramError> {
        let ca: CollectionAuthorityRecord = try_from_slice_checked(
            b,
            Key::CollectionAuthorityRecord,
            COLLECTION_AUTHORITY_RECORD_SIZE,
        )?;
        Ok(ca)
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum CollectionDetails {
    V1 { size: u64 },
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
        let expected_data = CollectionAuthorityRecord::default();

        let mut buf = Vec::new();
        expected_data.serialize(&mut buf).unwrap();
        CollectionAuthorityRecord::pad_length(&mut buf).unwrap();

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

        let data = CollectionAuthorityRecord::from_account_info(&account_info).unwrap();
        assert_eq!(data.key, Key::CollectionAuthorityRecord);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn deserializing_wrong_account_type_fails() {
        let wrong_type = UseAuthorityRecord::default();

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

        let error = CollectionAuthorityRecord::from_account_info(&account_info).unwrap_err();
        assert_eq!(error, MetadataError::DataTypeMismatch.into());
    }
}
