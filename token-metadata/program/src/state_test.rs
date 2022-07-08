#![cfg(test)]
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_sdk::{signature::Keypair, signer::Signer};

use crate::{
    deser::tests::{expected_pesky_metadata, pesky_data},
    error::MetadataError,
    state::{
        CollectionAuthorityRecord, Edition, EditionMarker, Key, MasterEditionV2, Metadata,
        UseAuthorityRecord, MAX_METADATA_LEN,
    },
    ID,
};
pub use crate::{state::Creator, utils::puff_out_data_fields};

#[cfg(test)]
mod metadata {

    use crate::state::TokenMetadataAccount;

    use super::*;

    fn pad_metadata_length(metadata: &mut Vec<u8>) {
        let padding_length = MAX_METADATA_LEN - metadata.len();
        metadata.extend(vec![0; padding_length]);
    }

    #[test]
    fn successfully_deserialize_corrupted_metadata() {
        // This should be able to deserialize the corrupted metadata account successfully due to the custom BorshDeserilization
        // implementation for the Metadata struct.
        let expected_metadata = expected_pesky_metadata();
        let mut corrupted_data = pesky_data();

        let metadata: Metadata = Metadata::deserialize(&mut corrupted_data).unwrap();

        assert_eq!(metadata, expected_metadata);
    }

    #[test]
    fn successfully_deserialize_metadata() {
        let expected_metadata = expected_pesky_metadata();

        let mut buf = Vec::new();
        expected_metadata.serialize(&mut buf).unwrap();
        pad_metadata_length(&mut buf);

        let pubkey = Keypair::new().pubkey();
        let owner = &ID;
        let mut lamports = 1_000_000_000;
        let mut data = buf.clone();

        let md_account_info = AccountInfo::new(
            &pubkey,
            false,
            true,
            &mut lamports,
            &mut data,
            owner,
            false,
            1_000_000_000,
        );

        let md: Metadata = Metadata::from_account_info(&md_account_info).unwrap();
        assert_eq!(md.key, Key::MetadataV1);
        assert_eq!(md, expected_metadata);
    }

    #[test]
    fn fail_to_deserialize_metadata_with_wrong_owner() {
        let expected_metadata = expected_pesky_metadata();

        let mut buf = Vec::new();
        expected_metadata.serialize(&mut buf).unwrap();
        pad_metadata_length(&mut buf);

        let pubkey = Keypair::new().pubkey();
        let invalid_owner = Keypair::new().pubkey();
        let mut lamports = 1_000_000_000;
        let mut data = buf.clone();

        let md_account_info = AccountInfo::new(
            &pubkey,
            false,
            true,
            &mut lamports,
            &mut data,
            &invalid_owner,
            false,
            1_000_000_000,
        );

        // `from_account_info` should not succeed because this account is not owned
        // by `token-metadata` program.
        let error = Metadata::from_account_info::<Metadata>(&md_account_info).unwrap_err();
        assert_eq!(error, MetadataError::IncorrectOwner.into());
    }

    #[test]
    fn fail_to_deserialize_metadata_with_wrong_size() {
        let expected_metadata = expected_pesky_metadata();

        let mut buf = Vec::new();
        expected_metadata.serialize(&mut buf).unwrap();
        // No padding is added to the metadata so it's too short.

        let pubkey = Keypair::new().pubkey();
        let owner = ID;
        let mut lamports = 1_000_000_000;
        let mut data = buf.clone();

        let account_info = AccountInfo::new(
            &pubkey,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            1_000_000_000,
        );

        // `from_account_info` should not succeed because this account is not owned
        // by `token-metadata` program.
        let error = Metadata::from_account_info::<Metadata>(&account_info).unwrap_err();
        assert_eq!(error, MetadataError::DataTypeMismatch.into());
    }

    #[test]
    fn fail_to_deserialize_master_edition_into_metadata() {
        let master_edition = MasterEditionV2 {
            key: Key::MasterEditionV2,
            supply: 0,
            max_supply: Some(0),
        };
        let mut buf = Vec::new();
        master_edition.serialize(&mut buf).unwrap();

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

        let err = Metadata::from_account_info::<Metadata>(&account_info).unwrap_err();
        assert_eq!(err, MetadataError::DataTypeMismatch.into());
    }

    #[test]
    fn fail_to_deserialize_edition_into_metadata() {
        let parent = Keypair::new().pubkey();
        let edition = 1;

        let edition = Edition {
            key: Key::EditionV1,
            parent,
            edition,
        };

        let mut buf = Vec::new();
        edition.serialize(&mut buf).unwrap();

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

        let err = Metadata::from_account_info::<Metadata>(&account_info).unwrap_err();
        assert_eq!(err, MetadataError::DataTypeMismatch.into());
    }

    #[test]
    fn fail_to_deserialize_use_authority_record_into_metadata() {
        let use_record = UseAuthorityRecord {
            key: Key::UseAuthorityRecord,
            allowed_uses: 14,
            bump: 255,
        };

        let mut buf = Vec::new();
        use_record.serialize(&mut buf).unwrap();

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

        let err = Metadata::from_account_info::<Metadata>(&account_info).unwrap_err();
        assert_eq!(err, MetadataError::DataTypeMismatch.into());
    }

    #[test]
    fn fail_to_deserialize_collection_authority_record_into_metadata() {
        let collection_record = CollectionAuthorityRecord {
            key: Key::CollectionAuthorityRecord,
            bump: 255,
        };

        let mut buf = Vec::new();
        collection_record.serialize(&mut buf).unwrap();

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

        let err = Metadata::from_account_info::<Metadata>(&account_info).unwrap_err();
        assert_eq!(err, MetadataError::DataTypeMismatch.into());
    }

    #[test]
    fn fail_to_deserialize_edition_marker_into_metadata() {
        let edition_marker = EditionMarker {
            key: Key::EditionMarker,
            ledger: [0; 31],
        };

        let mut buf = Vec::new();
        edition_marker.serialize(&mut buf).unwrap();

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

        let err = Metadata::from_account_info::<Metadata>(&account_info).unwrap_err();
        assert_eq!(err, MetadataError::DataTypeMismatch.into());
    }
}

mod master_edition {
    use crate::state::TokenMetadataAccount;

    use super::*;

    #[test]
    fn successfully_deserialize() {
        let expected_data = MasterEditionV2::default();

        let mut buf = Vec::new();
        expected_data.serialize(&mut buf).unwrap();
        MasterEditionV2::pad_length(&mut buf).unwrap();

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

        let data = MasterEditionV2::from_account_info::<MasterEditionV2>(&account_info).unwrap();
        assert_eq!(data.key, Key::MasterEditionV2);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn deserializing_wrong_account_type_fails() {
        let wrong_type = Metadata::default();

        let mut buf = Vec::new();
        wrong_type.serialize(&mut buf).unwrap();
        Metadata::pad_length(&mut buf).unwrap();

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

        let error =
            MasterEditionV2::from_account_info::<MasterEditionV2>(&account_info).unwrap_err();
        assert_eq!(error, MetadataError::DataTypeMismatch.into());
    }
}

mod edition {
    use crate::state::TokenMetadataAccount;

    use super::*;

    #[test]
    fn successfully_deserialize_edition() {
        let expected_data = Edition::default();

        let mut buf = Vec::new();
        expected_data.serialize(&mut buf).unwrap();
        Edition::pad_length(&mut buf).unwrap();

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

        let data = Edition::from_account_info::<Edition>(&account_info).unwrap();
        assert_eq!(data.key, Key::EditionV1);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn deserializing_wrong_account_type_fails() {
        let wrong_type = Metadata::default();

        let mut buf = Vec::new();
        wrong_type.serialize(&mut buf).unwrap();
        Metadata::pad_length(&mut buf).unwrap();

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

        let error = Edition::from_account_info::<Edition>(&account_info).unwrap_err();
        assert_eq!(error, MetadataError::DataTypeMismatch.into());
    }
}

mod edition_marker {
    use crate::state::TokenMetadataAccount;

    use super::*;

    #[test]
    fn successfully_deserialize() {
        let expected_data = EditionMarker::default();

        let mut buf = Vec::new();
        expected_data.serialize(&mut buf).unwrap();
        EditionMarker::pad_length(&mut buf).unwrap();

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

        let data = EditionMarker::from_account_info::<EditionMarker>(&account_info).unwrap();
        assert_eq!(data.key, Key::EditionMarker);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn deserializing_wrong_account_type_fails() {
        let wrong_type = Metadata::default();

        let mut buf = Vec::new();
        wrong_type.serialize(&mut buf).unwrap();
        Metadata::pad_length(&mut buf).unwrap();

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

        let error = EditionMarker::from_account_info::<EditionMarker>(&account_info).unwrap_err();
        assert_eq!(error, MetadataError::DataTypeMismatch.into());
    }
}

mod use_authority_record {
    use crate::state::TokenMetadataAccount;

    use super::*;

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

        let data =
            UseAuthorityRecord::from_account_info::<UseAuthorityRecord>(&account_info).unwrap();
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

        let error =
            UseAuthorityRecord::from_account_info::<UseAuthorityRecord>(&account_info).unwrap_err();
        assert_eq!(error, MetadataError::DataTypeMismatch.into());
    }
}

mod collection_authority_record {
    use crate::state::TokenMetadataAccount;

    use super::*;

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

        let data = CollectionAuthorityRecord::from_account_info::<CollectionAuthorityRecord>(
            &account_info,
        )
        .unwrap();
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

        let error = CollectionAuthorityRecord::from_account_info::<CollectionAuthorityRecord>(
            &account_info,
        )
        .unwrap_err();
        assert_eq!(error, MetadataError::DataTypeMismatch.into());
    }
}
