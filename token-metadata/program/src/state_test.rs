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

        let metadata = Metadata::deserialize(&mut corrupted_data).unwrap();

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

        let md = Metadata::from_account_info(&md_account_info).unwrap();
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
        let error = Metadata::from_account_info(&md_account_info).unwrap_err();
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
        let error = Metadata::from_account_info(&account_info).unwrap_err();
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

        let err = Metadata::from_account_info(&account_info).unwrap_err();
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

        let err = Metadata::from_account_info(&account_info).unwrap_err();
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

        let err = Metadata::from_account_info(&account_info).unwrap_err();
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

        let err = Metadata::from_account_info(&account_info).unwrap_err();
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

        let err = Metadata::from_account_info(&account_info).unwrap_err();
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

        let data = MasterEditionV2::from_account_info(&account_info).unwrap();
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

        let error = MasterEditionV2::from_account_info(&account_info).unwrap_err();
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

        let data = Edition::from_account_info(&account_info).unwrap();
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

        let error = Edition::from_account_info(&account_info).unwrap_err();
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

        let data = EditionMarker::from_account_info(&account_info).unwrap();
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

        let error = EditionMarker::from_account_info(&account_info).unwrap_err();
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

mod collection_authority_record {
    use crate::state::{
        EscrowConstraint, EscrowConstraintModel, EscrowConstraintType, TokenMetadataAccount,
    };

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

    #[test]
    fn test_escrow_constraints_model_len() {
        let ect_none = EscrowConstraintType::None;
        let ect_collection = EscrowConstraintType::Collection(Keypair::new().pubkey());
        let ect_tokens = EscrowConstraintType::tokens_from_slice(&[
            Keypair::new().pubkey(),
            Keypair::new().pubkey(),
            Keypair::new().pubkey(),
            Keypair::new().pubkey(),
        ]);

        let mut buf_ect_none = Vec::new();
        let mut buf_ect_collection = Vec::new();
        let mut buf_ect_tokens = Vec::new();

        ect_none.serialize(&mut buf_ect_none).unwrap();
        ect_collection.serialize(&mut buf_ect_collection).unwrap();
        ect_tokens.serialize(&mut buf_ect_tokens).unwrap();

        assert_eq!(
            ect_none.try_len().unwrap(),
            buf_ect_none.len(),
            "EscrowConstraintType::None length is not equal to serialized length"
        );

        assert_eq!(
            ect_collection.try_len().unwrap(),
            buf_ect_collection.len(),
            "EscrowConstraintType::Collection length is not equal to serialized length"
        );

        assert_eq!(
            ect_tokens.try_len().unwrap(),
            buf_ect_tokens.len(),
            "EscrowConstraintType::tokens length is not equal to serialized length"
        );

        let escrow_constraint_none = EscrowConstraint {
            name: "test".to_string(),
            constraint_type: ect_none,
            token_limit: 1,
        };

        let escrow_constraint_collection = EscrowConstraint {
            name: "test".to_string(),
            constraint_type: ect_collection,
            token_limit: 1,
        };

        let escrow_constraint_tokens = EscrowConstraint {
            name: "test".to_string(),
            constraint_type: ect_tokens,
            token_limit: 1,
        };

        let mut buf_escrow_constraint_none = Vec::new();
        let mut buf_escrow_constraint_collection = Vec::new();
        let mut buf_escrow_constraint_tokens = Vec::new();

        escrow_constraint_none
            .serialize(&mut buf_escrow_constraint_none)
            .unwrap();

        escrow_constraint_collection
            .serialize(&mut buf_escrow_constraint_collection)
            .unwrap();

        escrow_constraint_tokens
            .serialize(&mut buf_escrow_constraint_tokens)
            .unwrap();

        assert_eq!(
            escrow_constraint_none.try_len().unwrap(),
            buf_escrow_constraint_none.len(),
            "EscrowConstraint::None length is not equal to serialized length"
        );

        assert_eq!(
            escrow_constraint_collection.try_len().unwrap(),
            buf_escrow_constraint_collection.len(),
            "EscrowConstraint::Collection length is not equal to serialized length"
        );

        assert_eq!(
            escrow_constraint_tokens.try_len().unwrap(),
            buf_escrow_constraint_tokens.len(),
            "EscrowConstraint::tokens length is not equal to serialized length"
        );

        let escrow_constraints_model = EscrowConstraintModel {
            key: Key::EscrowConstraintModel,
            name: "test".to_string(),
            count: 0,
            update_authority: Keypair::new().pubkey(),
            creator: Keypair::new().pubkey(),
            constraints: vec![
                escrow_constraint_none,
                escrow_constraint_collection,
                escrow_constraint_tokens,
            ],
        };

        let mut buf_escrow_constraints_model = Vec::new();

        escrow_constraints_model
            .serialize(&mut buf_escrow_constraints_model)
            .unwrap();

        assert_eq!(
            escrow_constraints_model.try_len().unwrap(),
            buf_escrow_constraints_model.len(),
            "EscrowConstraintModel length is not equal to serialized length"
        );
    }

    #[test]
    fn test_validate_constraint() {
        let keypair_1 = Keypair::new();
        let keypair_2 = Keypair::new();
        let keypair_3 = Keypair::new();

        let ec_none = EscrowConstraint {
            name: "test".to_string(),
            constraint_type: EscrowConstraintType::None,
            token_limit: 1,
        };

        let ec_collection = EscrowConstraint {
            name: "test".to_string(),
            constraint_type: EscrowConstraintType::Collection(keypair_1.pubkey()),
            token_limit: 1,
        };

        let ec_tokens = EscrowConstraint {
            name: "test".to_string(),
            constraint_type: EscrowConstraintType::tokens_from_slice(&[
                keypair_2.pubkey(),
                keypair_3.pubkey(),
            ]),

            token_limit: 1,
        };
        let escrow_constraints_model = EscrowConstraintModel {
            key: Key::EscrowConstraintModel,
            name: "test".to_string(),
            count: 0,
            update_authority: Keypair::new().pubkey(),
            creator: Keypair::new().pubkey(),
            constraints: vec![ec_none, ec_collection, ec_tokens],
        };

        escrow_constraints_model
            .validate_at(&keypair_1.pubkey(), 0)
            .expect("None constraint failed");

        escrow_constraints_model
            .validate_at(&keypair_1.pubkey(), 1)
            .expect("Collection constraint failed");

        escrow_constraints_model
            .validate_at(&keypair_2.pubkey(), 1)
            .expect_err("Collection constraint failed");

        escrow_constraints_model
            .validate_at(&keypair_2.pubkey(), 2)
            .expect("Tokens constraint failed");

        escrow_constraints_model
            .validate_at(&keypair_1.pubkey(), 2)
            .expect_err("Tokens constraint failed");
    }
}
