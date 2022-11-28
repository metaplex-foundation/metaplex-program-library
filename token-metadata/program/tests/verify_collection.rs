#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::{
    error::MetadataError,
    pda::find_collection_authority_account,
    state::{
        Collection, CollectionAuthorityRecord, Key, UseMethod, Uses,
        COLLECTION_AUTHORITY_RECORD_SIZE, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH,
    },
    utils::puffed_out_string,
};
use num_traits::FromPrimitive;
use solana_program::{borsh::try_from_slice_unchecked, native_token::LAMPORTS_PER_SOL};
use solana_program_test::*;
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
};
use utils::*;
mod verify_collection {

    use mpl_token_metadata::state::{CollectionAuthorityRecord, COLLECTION_AUTHORITY_RECORD_SIZE};
    use solana_program::borsh::try_from_slice_unchecked;
    use solana_sdk::transaction::Transaction;

    use super::*;
    #[tokio::test]
    async fn success_verify_collection() {
        let mut context = program_test().start_with_context().await;

        let test_collection = Metadata::new();
        test_collection
            .create_v2_default(&mut context)
            .await
            .unwrap();
        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let name = "Test".to_string();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();
        let test_metadata = Metadata::new();

        let puffed_name = puffed_out_string(&name, MAX_NAME_LENGTH);
        let puffed_symbol = puffed_out_string(&symbol, MAX_SYMBOL_LENGTH);
        let puffed_uri = puffed_out_string(&uri, MAX_URI_LENGTH);

        let uses = Some(Uses {
            total: 1,
            remaining: 1,
            use_method: UseMethod::Single,
        });
        test_metadata
            .create_v2(
                &mut context,
                name,
                symbol,
                uri,
                None,
                10,
                false,
                Some(Collection {
                    key: test_collection.mint.pubkey(),
                    verified: false,
                }),
                uses.to_owned(),
            )
            .await
            .unwrap();

        let metadata = test_metadata.get_data(&mut context).await;

        assert_eq!(metadata.data.name, puffed_name);
        assert_eq!(metadata.data.symbol, puffed_symbol);
        assert_eq!(metadata.data.uri, puffed_uri);
        assert_eq!(metadata.data.seller_fee_basis_points, 10);
        assert_eq!(metadata.data.creators, None);
        assert_eq!(metadata.uses, uses.to_owned());

        assert_eq!(
            metadata.collection.to_owned().unwrap().key,
            test_collection.mint.pubkey()
        );
        assert!(!metadata.collection.unwrap().verified);

        assert!(!metadata.primary_sale_happened);
        assert!(!metadata.is_mutable);
        assert_eq!(metadata.mint, test_metadata.mint.pubkey());
        assert_eq!(metadata.update_authority, context.payer.pubkey());
        assert_eq!(metadata.key, Key::MetadataV1);
        let kpbytes = &context.payer;
        let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();
        test_metadata
            .verify_collection(
                &mut context,
                test_collection.pubkey,
                &kp,
                test_collection.mint.pubkey(),
                collection_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap();

        let metadata_after = test_metadata.get_data(&mut context).await;
        assert_eq!(
            metadata_after.collection.to_owned().unwrap().key,
            test_collection.mint.pubkey()
        );
        assert!(metadata_after.collection.unwrap().verified);
    }

    #[tokio::test]
    async fn fail_wrong_collection_from_authority() {
        let mut context = program_test().start_with_context().await;

        let test_collection = Metadata::new();
        test_collection
            .create_v2_default(&mut context)
            .await
            .unwrap();

        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let test_collection2 = Metadata::new();
        test_collection2
            .create_v2_default(&mut context)
            .await
            .unwrap();

        let collection_master_edition_account2 = MasterEditionV2::new(&test_collection2);
        collection_master_edition_account2
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let name = "Test".to_string();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();
        let test_metadata = Metadata::new();

        let puffed_name = puffed_out_string(&name, MAX_NAME_LENGTH);
        let puffed_symbol = puffed_out_string(&symbol, MAX_SYMBOL_LENGTH);
        let puffed_uri = puffed_out_string(&uri, MAX_URI_LENGTH);

        let uses = Some(Uses {
            total: 1,
            remaining: 1,
            use_method: UseMethod::Single,
        });
        test_metadata
            .create_v2(
                &mut context,
                name,
                symbol,
                uri,
                None,
                10,
                false,
                Some(Collection {
                    key: test_collection.mint.pubkey(),
                    verified: false,
                }),
                uses.to_owned(),
            )
            .await
            .unwrap();

        let metadata = test_metadata.get_data(&mut context).await;

        assert_eq!(metadata.data.name, puffed_name);
        assert_eq!(metadata.data.symbol, puffed_symbol);
        assert_eq!(metadata.data.uri, puffed_uri);
        assert_eq!(metadata.data.seller_fee_basis_points, 10);
        assert_eq!(metadata.data.creators, None);
        assert_eq!(metadata.uses, uses.to_owned());

        assert_eq!(
            metadata.collection.to_owned().unwrap().key,
            test_collection.mint.pubkey()
        );
        assert!(!metadata.collection.unwrap().verified);

        assert!(!metadata.primary_sale_happened);
        assert!(!metadata.is_mutable);
        assert_eq!(metadata.mint, test_metadata.mint.pubkey());
        assert_eq!(metadata.update_authority, context.payer.pubkey());
        assert_eq!(metadata.key, Key::MetadataV1);
        let kpbytes = &context.payer;
        let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();
        let err = test_metadata
            .verify_collection(
                &mut context,
                test_collection.pubkey,
                &kp,
                test_collection2.mint.pubkey(),
                collection_master_edition_account2.pubkey,
                None,
            )
            .await
            .unwrap_err();
        assert_custom_error!(err, MetadataError::CollectionNotFound);
    }

    #[tokio::test]
    async fn fail_no_collection_nft_token_standard() {
        let mut context = program_test().start_with_context().await;

        let test_collection = Metadata::new();
        test_collection
            .create_v2_default(&mut context)
            .await
            .unwrap();

        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create(&mut context, Some(0))
            .await
            .unwrap();

        let name = "Test".to_string();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();
        let test_metadata = Metadata::new();
        let uses = Some(Uses {
            total: 1,
            remaining: 1,
            use_method: UseMethod::Single,
        });
        test_metadata
            .create_v2(
                &mut context,
                name,
                symbol,
                uri,
                None,
                10,
                false,
                Some(Collection {
                    key: test_collection.mint.pubkey(),
                    verified: false,
                }),
                uses.to_owned(),
            )
            .await
            .unwrap();

        let kpbytes = &context.payer;
        let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();
        let err = test_metadata
            .verify_collection(
                &mut context,
                test_collection.pubkey,
                &kp,
                test_collection.mint.pubkey(),
                collection_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap_err();
        assert_custom_error!(err, MetadataError::CollectionMustBeAUniqueMasterEdition);
        let metadata_after = test_metadata.get_data(&mut context).await;
        assert_eq!(
            metadata_after.collection.to_owned().unwrap().key,
            test_collection.mint.pubkey()
        );
        assert!(!metadata_after.collection.unwrap().verified);
    }

    #[tokio::test]
    async fn fail_non_unique_master_edition() {
        let mut context = program_test().start_with_context().await;

        let test_collection = Metadata::new();
        test_collection
            .create_v2_default(&mut context)
            .await
            .unwrap();

        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create(&mut context, Some(1))
            .await
            .unwrap();

        let name = "Test".to_string();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();
        let test_metadata = Metadata::new();
        let uses = Some(Uses {
            total: 1,
            remaining: 1,
            use_method: UseMethod::Single,
        });
        test_metadata
            .create_v2(
                &mut context,
                name,
                symbol,
                uri,
                None,
                10,
                false,
                Some(Collection {
                    key: test_collection.mint.pubkey(),
                    verified: false,
                }),
                uses.to_owned(),
            )
            .await
            .unwrap();

        let kpbytes = &context.payer;
        let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();
        let err = test_metadata
            .verify_collection(
                &mut context,
                test_collection.pubkey,
                &kp,
                test_collection.mint.pubkey(),
                collection_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap_err();
        assert_custom_error!(err, MetadataError::CollectionMustBeAUniqueMasterEdition);
        let metadata_after = test_metadata.get_data(&mut context).await;
        assert_eq!(
            metadata_after.collection.to_owned().unwrap().key,
            test_collection.mint.pubkey()
        );
        assert!(!metadata_after.collection.unwrap().verified);
    }

    #[tokio::test]
    async fn fail_no_master_edition() {
        let mut context = program_test().start_with_context().await;

        let test_collection = Metadata::new();
        test_collection
            .create_v2_default(&mut context)
            .await
            .unwrap();

        let name = "Test".to_string();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();
        let test_metadata = Metadata::new();

        let uses = Some(Uses {
            total: 1,
            remaining: 1,
            use_method: UseMethod::Single,
        });
        test_metadata
            .create_v2(
                &mut context,
                name,
                symbol,
                uri,
                None,
                10,
                false,
                Some(Collection {
                    key: test_collection.mint.pubkey(),
                    verified: false,
                }),
                uses.to_owned(),
            )
            .await
            .unwrap();

        let kpbytes = &context.payer;
        let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();
        let err = test_metadata
            .verify_collection(
                &mut context,
                test_collection.pubkey,
                &kp,
                test_collection.mint.pubkey(),
                test_collection.pubkey,
                None,
            )
            .await
            .unwrap_err();
        assert_custom_error!(err, MetadataError::CollectionMasterEditionAccountInvalid);
        let metadata_after = test_metadata.get_data(&mut context).await;
        assert_eq!(
            metadata_after.collection.to_owned().unwrap().key,
            test_collection.mint.pubkey()
        );
        assert!(!metadata_after.collection.unwrap().verified);
    }

    #[tokio::test]
    async fn fail_collection_authority_mismatch() {
        let mut context = program_test().start_with_context().await;
        let collection_authority = Keypair::new();

        let test_collection = Metadata::new();
        test_collection
            .create_v2_default(&mut context)
            .await
            .unwrap();

        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let name = "Test".to_string();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();
        let test_metadata = Metadata::new();

        let uses = Some(Uses {
            total: 1,
            remaining: 1,
            use_method: UseMethod::Single,
        });
        test_metadata
            .create_v2(
                &mut context,
                name,
                symbol,
                uri,
                None,
                10,
                false,
                Some(Collection {
                    key: test_collection.mint.pubkey(),
                    verified: false,
                }),
                uses.to_owned(),
            )
            .await
            .unwrap();

        let err = test_metadata
            .verify_collection(
                &mut context,
                test_collection.pubkey,
                &collection_authority,
                test_collection.mint.pubkey(),
                collection_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap_err();
        assert_custom_error!(err, MetadataError::InvalidCollectionUpdateAuthority);
        let metadata_after = test_metadata.get_data(&mut context).await;
        assert_eq!(
            metadata_after.collection.to_owned().unwrap().key,
            test_collection.mint.pubkey()
        );
        assert!(!metadata_after.collection.unwrap().verified);
    }

    #[tokio::test]
    async fn success() {
        let mut context = program_test().start_with_context().await;

        let test_collection = Metadata::new();
        test_collection
            .create_v2_default(&mut context)
            .await
            .unwrap();

        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let name = "Test".to_string();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();
        let test_metadata = Metadata::new();

        let puffed_name = puffed_out_string(&name, MAX_NAME_LENGTH);
        let puffed_symbol = puffed_out_string(&symbol, MAX_SYMBOL_LENGTH);
        let puffed_uri = puffed_out_string(&uri, MAX_URI_LENGTH);

        let uses = Some(Uses {
            total: 1,
            remaining: 1,
            use_method: UseMethod::Single,
        });
        test_metadata
            .create_v2(
                &mut context,
                name,
                symbol,
                uri,
                None,
                10,
                false,
                Some(Collection {
                    key: test_collection.mint.pubkey(),
                    verified: false,
                }),
                uses.to_owned(),
            )
            .await
            .unwrap();

        let metadata = test_metadata.get_data(&mut context).await;

        assert_eq!(metadata.data.name, puffed_name);
        assert_eq!(metadata.data.symbol, puffed_symbol);
        assert_eq!(metadata.data.uri, puffed_uri);
        assert_eq!(metadata.data.seller_fee_basis_points, 10);
        assert_eq!(metadata.data.creators, None);
        assert_eq!(metadata.uses, uses.to_owned());

        assert_eq!(
            metadata.collection.to_owned().unwrap().key,
            test_collection.mint.pubkey()
        );
        assert!(!metadata.collection.unwrap().verified);

        assert!(!metadata.primary_sale_happened);
        assert!(!metadata.is_mutable);
        assert_eq!(metadata.mint, test_metadata.mint.pubkey());
        assert_eq!(metadata.update_authority, context.payer.pubkey());
        assert_eq!(metadata.key, Key::MetadataV1);
        let kpbytes = &context.payer;
        let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();
        test_metadata
            .verify_collection(
                &mut context,
                test_collection.pubkey,
                &kp,
                test_collection.mint.pubkey(),
                collection_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap();

        let metadata_after = test_metadata.get_data(&mut context).await;
        assert_eq!(
            metadata_after.collection.to_owned().unwrap().key,
            test_collection.mint.pubkey()
        );
        assert!(metadata_after.collection.unwrap().verified);
    }

    #[tokio::test]
    async fn success_verify_collection_with_authority() {
        let mut context = program_test().start_with_context().await;
        let new_collection_authority = Keypair::new();
        let test_collection = Metadata::new();
        test_collection
            .create_v2_default(&mut context)
            .await
            .unwrap();

        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let name = "Test".to_string();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();
        let test_metadata = Metadata::new();
        test_metadata
            .create_v2(
                &mut context,
                name,
                symbol,
                uri,
                None,
                10,
                false,
                Some(Collection {
                    key: test_collection.mint.pubkey(),
                    verified: false,
                }),
                None,
            )
            .await
            .unwrap();

        let metadata = test_metadata.get_data(&mut context).await;
        assert_eq!(
            metadata.collection.to_owned().unwrap().key,
            test_collection.mint.pubkey()
        );
        assert!(!metadata.collection.unwrap().verified);
        let (record, _) = find_collection_authority_account(
            &test_collection.mint.pubkey(),
            &new_collection_authority.pubkey(),
        );
        let ix = mpl_token_metadata::instruction::approve_collection_authority(
            mpl_token_metadata::id(),
            record,
            new_collection_authority.pubkey(),
            context.payer.pubkey(),
            context.payer.pubkey(),
            test_collection.pubkey,
            test_collection.mint.pubkey(),
        );

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let record_account = get_account(&mut context, &record).await;
        let record_data: CollectionAuthorityRecord =
            try_from_slice_unchecked(&record_account.data).unwrap();
        assert_eq!(record_data.key, Key::CollectionAuthorityRecord);

        test_metadata
            .verify_collection(
                &mut context,
                test_collection.pubkey,
                &new_collection_authority,
                test_collection.mint.pubkey(),
                collection_master_edition_account.pubkey,
                Some(record),
            )
            .await
            .unwrap();

        let metadata_after = test_metadata.get_data(&mut context).await;
        assert_eq!(
            metadata_after.collection.to_owned().unwrap().key,
            test_collection.mint.pubkey()
        );
        assert!(metadata_after.collection.unwrap().verified);

        test_metadata
            .unverify_collection(
                &mut context,
                test_collection.pubkey,
                &new_collection_authority,
                test_collection.mint.pubkey(),
                collection_master_edition_account.pubkey,
                Some(record),
            )
            .await
            .unwrap();
        let metadata_after_unverify = test_metadata.get_data(&mut context).await;
        assert!(!metadata_after_unverify.collection.unwrap().verified);
    }

    #[tokio::test]
    async fn success_set_and_verify_collection_with_authority() {
        let mut context = program_test().start_with_context().await;
        let new_collection_authority = Keypair::new();
        let test_collection = Metadata::new();
        test_collection
            .create_v2_default(&mut context)
            .await
            .unwrap();

        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let test_metadata = Metadata::new();
        test_metadata.create_v2_default(&mut context).await.unwrap();

        let metadata = test_metadata.get_data(&mut context).await;
        assert!(metadata.collection.is_none());
        let update_authority = context.payer.pubkey();
        let (record, _) = find_collection_authority_account(
            &test_collection.mint.pubkey(),
            &new_collection_authority.pubkey(),
        );
        let ix = mpl_token_metadata::instruction::approve_collection_authority(
            mpl_token_metadata::id(),
            record,
            new_collection_authority.pubkey(),
            update_authority,
            context.payer.pubkey(),
            test_collection.pubkey,
            test_collection.mint.pubkey(),
        );

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let record_account = get_account(&mut context, &record).await;
        let record_data: CollectionAuthorityRecord =
            try_from_slice_unchecked(&record_account.data).unwrap();
        assert_eq!(record_data.key, Key::CollectionAuthorityRecord);

        test_metadata
            .set_and_verify_collection(
                &mut context,
                test_collection.pubkey,
                &new_collection_authority,
                update_authority,
                test_collection.mint.pubkey(),
                collection_master_edition_account.pubkey,
                Some(record),
            )
            .await
            .unwrap();

        let metadata_after = test_metadata.get_data(&mut context).await;
        assert_eq!(
            metadata_after.collection.to_owned().unwrap().key,
            test_collection.mint.pubkey()
        );
        assert!(metadata_after.collection.unwrap().verified);

        test_metadata
            .unverify_collection(
                &mut context,
                test_collection.pubkey,
                &new_collection_authority,
                test_collection.mint.pubkey(),
                collection_master_edition_account.pubkey,
                Some(record),
            )
            .await
            .unwrap();
        let metadata_after_unverify = test_metadata.get_data(&mut context).await;
        assert!(!metadata_after_unverify.collection.unwrap().verified);
    }

    #[tokio::test]
    async fn success_set_and_verify_collection_with_authority_and_revoke_as_delegate() {
        let mut context = program_test().start_with_context().await;
        let new_collection_authority = Keypair::new();
        airdrop(&mut context, &new_collection_authority.pubkey(), 10000000)
            .await
            .unwrap();

        let test_collection = Metadata::new();
        test_collection
            .create_v2_default(&mut context)
            .await
            .unwrap();

        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let test_metadata = Metadata::new();
        test_metadata.create_v2_default(&mut context).await.unwrap();

        let metadata = test_metadata.get_data(&mut context).await;
        assert!(metadata.collection.is_none());
        let update_authority = context.payer.pubkey();
        let (record, _) = find_collection_authority_account(
            &test_collection.mint.pubkey(),
            &new_collection_authority.pubkey(),
        );
        let ix = mpl_token_metadata::instruction::approve_collection_authority(
            mpl_token_metadata::id(),
            record,
            new_collection_authority.pubkey(),
            update_authority,
            context.payer.pubkey(),
            test_collection.pubkey,
            test_collection.mint.pubkey(),
        );

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let record_account = get_account(&mut context, &record).await;
        let record_data: CollectionAuthorityRecord =
            try_from_slice_unchecked(&record_account.data).unwrap();
        assert_eq!(record_data.key, Key::CollectionAuthorityRecord);

        test_metadata
            .set_and_verify_collection(
                &mut context,
                test_collection.pubkey,
                &new_collection_authority,
                update_authority,
                test_collection.mint.pubkey(),
                collection_master_edition_account.pubkey,
                Some(record),
            )
            .await
            .unwrap();

        let metadata_after = test_metadata.get_data(&mut context).await;
        assert_eq!(
            metadata_after.collection.to_owned().unwrap().key,
            test_collection.mint.pubkey()
        );
        assert!(metadata_after.collection.unwrap().verified);

        let ix_revoke = mpl_token_metadata::instruction::revoke_collection_authority(
            mpl_token_metadata::id(),
            record,
            new_collection_authority.pubkey(),
            new_collection_authority.pubkey(),
            test_collection.pubkey,
            test_collection.mint.pubkey(),
        );

        let tx_revoke = Transaction::new_signed_with_payer(
            &[ix_revoke],
            Some(&new_collection_authority.pubkey()),
            &[&new_collection_authority],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(tx_revoke)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn fail_verify_collection_with_authority() {
        let mut context = program_test().start_with_context().await;
        let new_collection_authority = Keypair::new();
        let test_collection = Metadata::new();
        test_collection
            .create_v2_default(&mut context)
            .await
            .unwrap();

        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let name = "Test".to_string();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();
        let test_metadata = Metadata::new();
        let uses = Some(Uses {
            total: 1,
            remaining: 1,
            use_method: UseMethod::Single,
        });
        test_metadata
            .create_v2(
                &mut context,
                name,
                symbol,
                uri,
                None,
                10,
                false,
                Some(Collection {
                    key: test_collection.mint.pubkey(),
                    verified: false,
                }),
                uses.to_owned(),
            )
            .await
            .unwrap();

        let metadata = test_metadata.get_data(&mut context).await;
        assert_eq!(
            metadata.collection.to_owned().unwrap().key,
            test_collection.mint.pubkey()
        );
        assert!(!metadata.collection.unwrap().verified);
        let (record, _) = find_collection_authority_account(
            &test_collection.mint.pubkey(),
            &new_collection_authority.pubkey(),
        );
        let ix = mpl_token_metadata::instruction::approve_collection_authority(
            mpl_token_metadata::id(),
            record,
            new_collection_authority.pubkey(),
            context.payer.pubkey(),
            context.payer.pubkey(),
            test_collection.pubkey,
            test_collection.mint.pubkey(),
        );

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let account_before = context
            .banks_client
            .get_account(record)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(account_before.data.len(), COLLECTION_AUTHORITY_RECORD_SIZE);

        let ixrevoke = mpl_token_metadata::instruction::revoke_collection_authority(
            mpl_token_metadata::id(),
            record,
            new_collection_authority.pubkey(),
            context.payer.pubkey(),
            test_collection.pubkey,
            test_collection.mint.pubkey(),
        );

        let txrevoke = Transaction::new_signed_with_payer(
            &[ixrevoke],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(txrevoke)
            .await
            .unwrap();

        let account_after_none = context
            .banks_client
            .get_account(record)
            .await
            .unwrap()
            .is_none();
        assert!(account_after_none);

        let err = test_metadata
            .verify_collection(
                &mut context,
                test_collection.pubkey,
                &new_collection_authority,
                test_collection.mint.pubkey(),
                collection_master_edition_account.pubkey,
                Some(record),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidCollectionUpdateAuthority);
        let metadata_after = test_metadata.get_data(&mut context).await;
        assert!(!metadata_after.collection.unwrap().verified);
    }

    #[tokio::test]
    async fn fail_set_and_verify_collection_with_authority_and_revoke_as_wrong_signer() {
        let mut context = program_test().start_with_context().await;
        let new_collection_authority = Keypair::new();
        let incorrect_revoke_authority = Keypair::new();
        airdrop(&mut context, &incorrect_revoke_authority.pubkey(), 10000000)
            .await
            .unwrap();

        let test_collection = Metadata::new();
        test_collection
            .create_v2_default(&mut context)
            .await
            .unwrap();

        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let test_metadata = Metadata::new();
        test_metadata.create_v2_default(&mut context).await.unwrap();
        let metadata = test_metadata.get_data(&mut context).await;
        assert!(metadata.collection.is_none());
        let update_authority = context.payer.pubkey();
        let (record, _) = find_collection_authority_account(
            &test_collection.mint.pubkey(),
            &new_collection_authority.pubkey(),
        );
        let ix = mpl_token_metadata::instruction::approve_collection_authority(
            mpl_token_metadata::id(),
            record,
            new_collection_authority.pubkey(),
            update_authority,
            context.payer.pubkey(),
            test_collection.pubkey,
            test_collection.mint.pubkey(),
        );

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let record_account = get_account(&mut context, &record).await;
        let record_data: CollectionAuthorityRecord =
            try_from_slice_unchecked(&record_account.data).unwrap();
        assert_eq!(record_data.key, Key::CollectionAuthorityRecord);

        test_metadata
            .set_and_verify_collection(
                &mut context,
                test_collection.pubkey,
                &new_collection_authority,
                update_authority,
                test_collection.mint.pubkey(),
                collection_master_edition_account.pubkey,
                Some(record),
            )
            .await
            .unwrap();

        let metadata_after = test_metadata.get_data(&mut context).await;
        assert_eq!(
            metadata_after.collection.to_owned().unwrap().key,
            test_collection.mint.pubkey()
        );
        assert!(metadata_after.collection.unwrap().verified);

        test_metadata
            .unverify_collection(
                &mut context,
                test_collection.pubkey,
                &new_collection_authority,
                test_collection.mint.pubkey(),
                collection_master_edition_account.pubkey,
                Some(record),
            )
            .await
            .unwrap();
        let metadata_after_unverify = test_metadata.get_data(&mut context).await;
        assert!(!metadata_after_unverify.collection.unwrap().verified);

        let ix_revoke = mpl_token_metadata::instruction::revoke_collection_authority(
            mpl_token_metadata::id(),
            record,
            new_collection_authority.pubkey(),
            incorrect_revoke_authority.pubkey(),
            test_collection.pubkey,
            test_collection.mint.pubkey(),
        );

        let tx_revoke = Transaction::new_signed_with_payer(
            &[ix_revoke],
            Some(&incorrect_revoke_authority.pubkey()),
            &[&incorrect_revoke_authority],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx_revoke)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::RevokeCollectionAuthoritySignerIncorrect);
    }
}

#[tokio::test]
async fn fail_verify_collection_invalid_owner() {
    let mut context = program_test().start_with_context().await;

    let test_collection = Metadata::new();
    test_collection
        .create_v2_default(&mut context)
        .await
        .unwrap();

    let collection_master_edition_account = MasterEditionV2::new(&test_collection);
    collection_master_edition_account
        .create_v3(&mut context, Some(0))
        .await
        .unwrap();

    let name = "Test".to_string();
    let symbol = "TST".to_string();
    let uri = "uri".to_string();
    let test_metadata = Metadata::new();

    let uses = Some(Uses {
        total: 1,
        remaining: 1,
        use_method: UseMethod::Single,
    });

    test_metadata
        .create_v2(
            &mut context,
            name,
            symbol,
            uri,
            None,
            10,
            false,
            Some(Collection {
                key: test_collection.mint.pubkey(),
                verified: false,
            }),
            uses.to_owned(),
        )
        .await
        .unwrap();

    let kpbytes = &context.payer;
    let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();
    let fake_mint = Keypair::new();
    let err = test_metadata
        .verify_collection(
            &mut context,
            test_collection.pubkey,
            &kp,
            fake_mint.pubkey(),
            collection_master_edition_account.pubkey,
            None,
        )
        .await
        .unwrap_err();

    assert_custom_error!(err, MetadataError::IncorrectOwner);
}

#[tokio::test]
async fn fail_verify_collection_negative_cases() {
    let mut context = program_test().start_with_context().await;

    let test_collection = Metadata::new();
    test_collection
        .create_v2_default(&mut context)
        .await
        .unwrap();

    let test_collection_fake = Metadata::new();
    test_collection_fake
        .create_v2_default(&mut context)
        .await
        .unwrap();

    let fake_collection_master_edition_account = MasterEditionV2::new(&test_collection_fake);
    fake_collection_master_edition_account
        .create_v3(&mut context, Some(0))
        .await
        .unwrap();

    let name = "Test".to_string();
    let symbol = "TST".to_string();
    let uri = "uri".to_string();
    let test_metadata = Metadata::new();

    let uses = Some(Uses {
        total: 1,
        remaining: 1,
        use_method: UseMethod::Single,
    });

    test_metadata
        .create_v2(
            &mut context,
            name,
            symbol,
            uri,
            None,
            10,
            false,
            Some(Collection {
                key: test_collection.mint.pubkey(),
                verified: false,
            }),
            uses.to_owned(),
        )
        .await
        .unwrap();

    let kpbytes = &context.payer;
    let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();
    // Mismatch of collection mint and master edition
    let err = test_metadata
        .verify_collection(
            &mut context,
            test_collection.pubkey,
            &kp,
            fake_collection_master_edition_account.mint_pubkey,
            fake_collection_master_edition_account.pubkey,
            None,
        )
        .await
        .unwrap_err();
    assert_custom_error!(err, MetadataError::CollectionNotFound);
    // Mismatch master edition but correct mint
    let err = test_metadata
        .verify_collection(
            &mut context,
            test_collection.pubkey,
            &kp,
            test_collection.mint.pubkey(),
            fake_collection_master_edition_account.pubkey,
            None,
        )
        .await
        .unwrap_err();
    assert_custom_error!(err, MetadataError::CollectionMasterEditionAccountInvalid);
    // Random Edition account
    let key = Keypair::new();
    let err = test_metadata
        .verify_collection(
            &mut context,
            test_collection.pubkey,
            &kp,
            test_collection.mint.pubkey(),
            key.pubkey(),
            None,
        )
        .await
        .unwrap_err();
    assert_custom_error!(err, MetadataError::IncorrectOwner);
}

#[tokio::test]
async fn fail_invalid_collection_update_authority() {
    let mut context = program_test().start_with_context().await;

    let user_keypair = Keypair::new();

    let test_collection = Metadata::new();
    test_collection
        .create_v2_default(&mut context)
        .await
        .unwrap();

    let collection_master_edition_account = MasterEditionV2::new(&test_collection);
    collection_master_edition_account
        .create_v3(&mut context, Some(0))
        .await
        .unwrap();

    let user_nft = Metadata::new();
    user_nft.create_v2_default(&mut context).await.unwrap();

    let user_master_edition_account = MasterEditionV2::new(&user_nft);
    user_master_edition_account
        .create_v3(&mut context, Some(0))
        .await
        .unwrap();

    user_nft
        .change_update_authority(&mut context, user_keypair.pubkey())
        .await
        .unwrap();

    // Setup delegate
    let delegate_keypair = Keypair::new();

    let update_authority = context.payer.pubkey();
    let (record, _) = find_collection_authority_account(
        &test_collection.mint.pubkey(),
        &delegate_keypair.pubkey(),
    );

    let ix1 = mpl_token_metadata::instruction::approve_collection_authority(
        mpl_token_metadata::id(),
        record,
        delegate_keypair.pubkey(),
        update_authority,
        context.payer.pubkey(),
        test_collection.pubkey,
        test_collection.mint.pubkey(),
    );

    let tx1 = Transaction::new_signed_with_payer(
        &[ix1],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx1).await.unwrap();

    // Change update authority to match users keypair
    test_collection
        .change_update_authority(&mut context, user_keypair.pubkey())
        .await
        .unwrap();

    let err = user_nft
        .set_and_verify_collection(
            &mut context,
            test_collection.pubkey,
            &delegate_keypair,
            user_keypair.pubkey(),
            test_collection.mint.pubkey(),
            collection_master_edition_account.pubkey,
            Some(record),
        )
        .await
        .unwrap_err();

    assert_custom_error!(err, MetadataError::InvalidCollectionUpdateAuthority);
}

#[tokio::test]
async fn success_collection_authority_delegate_revoke() {
    let mut context = program_test().start_with_context().await;

    let test_collection = Metadata::new();
    test_collection
        .create_v2_default(&mut context)
        .await
        .unwrap();

    let collection_master_edition_account = MasterEditionV2::new(&test_collection);
    collection_master_edition_account
        .create_v3(&mut context, Some(0))
        .await
        .unwrap();

    let user_nft = Metadata::new();
    user_nft.create_v2_default(&mut context).await.unwrap();

    let user_master_edition_account = MasterEditionV2::new(&user_nft);
    user_master_edition_account
        .create_v3(&mut context, Some(0))
        .await
        .unwrap();

    // Setup delegate
    let delegate_keypair = Keypair::new();

    let (record, bump) = find_collection_authority_account(
        &test_collection.mint.pubkey(),
        &delegate_keypair.pubkey(),
    );

    let mut data = vec![0u8; 11];
    data[0] = 9; // key
    data[1] = bump; // bump

    let record_account = Account {
        lamports: LAMPORTS_PER_SOL,
        data,
        owner: mpl_token_metadata::ID,
        executable: false,
        rent_epoch: 1,
    };
    let record_account_shared_data: AccountSharedData = record_account.into();
    context.set_account(&record, &record_account_shared_data);

    let payer = context.payer.pubkey();

    let ix_revoke = mpl_token_metadata::instruction::revoke_collection_authority(
        mpl_token_metadata::id(),
        record,
        delegate_keypair.pubkey(),
        payer,
        test_collection.pubkey,
        test_collection.mint.pubkey(),
    );

    let tx_revoke = Transaction::new_signed_with_payer(
        &[ix_revoke],
        Some(&payer),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx_revoke)
        .await
        .unwrap();

    let ix = mpl_token_metadata::instruction::approve_collection_authority(
        mpl_token_metadata::id(),
        record,
        delegate_keypair.pubkey(),
        payer,
        payer,
        test_collection.pubkey,
        test_collection.mint.pubkey(),
    );

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    let record_account = get_account(&mut context, &record).await;
    let record_data: CollectionAuthorityRecord =
        try_from_slice_unchecked(&record_account.data).unwrap();
    assert_eq!(record_data.key, Key::CollectionAuthorityRecord);
    assert_eq!(record_data.update_authority, Some(payer));
    assert_eq!(record_account.data.len(), COLLECTION_AUTHORITY_RECORD_SIZE);
}
