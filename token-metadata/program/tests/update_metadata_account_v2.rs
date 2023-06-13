#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::{
    error::MetadataError,
    instruction,
    state::{
        Collection, Creator, DataV2, Key, UseMethod, Uses, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH,
        MAX_URI_LENGTH,
    },
    utils::puffed_out_string,
    ID,
};
use num_traits::FromPrimitive;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
};
use utils::*;

mod update_metadata_account_v2 {
    use mpl_token_metadata::pda::find_collection_authority_account;

    use super::*;

    #[tokio::test]
    async fn success_compatible() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();

        let puffed_symbol = puffed_out_string(&symbol, MAX_SYMBOL_LENGTH);
        let puffed_uri = puffed_out_string(&uri, MAX_URI_LENGTH);

        test_metadata.create_v3_default(&mut context).await.unwrap();

        let updated_name = "New Name".to_string();
        let puffed_updated_name = puffed_out_string(&updated_name, MAX_NAME_LENGTH);

        let creators = Some(vec![Creator {
            address: context.payer.pubkey(),
            verified: true,
            share: 100,
        }]);

        test_metadata
            .update_v2(
                &mut context,
                updated_name,
                symbol,
                uri,
                creators.clone(),
                10,
                false,
                Some(Collection {
                    key: test_metadata.pubkey,
                    verified: false,
                }),
                Some(Uses {
                    use_method: UseMethod::Multiple,
                    remaining: 5,
                    total: 10,
                }),
            )
            .await
            .unwrap();

        let metadata = test_metadata.get_data(&mut context).await;

        assert_eq!(metadata.data.name, puffed_updated_name,);
        assert_eq!(metadata.data.symbol, puffed_symbol);
        assert_eq!(metadata.data.uri, puffed_uri);
        assert_eq!(metadata.data.seller_fee_basis_points, 10);
        assert_eq!(metadata.data.creators, creators);

        assert!(!metadata.primary_sale_happened);
        assert!(!metadata.is_mutable);
        assert_eq!(metadata.mint, test_metadata.mint.pubkey());
        assert_eq!(metadata.update_authority, context.payer.pubkey());
        assert_eq!(metadata.key, Key::MetadataV1);
        assert_eq!(metadata.collection.unwrap().key, test_metadata.pubkey);
        assert_eq!(metadata.uses.unwrap().use_method, UseMethod::Multiple)
    }

    #[tokio::test]
    async fn success() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();

        let creators = Some(vec![Creator {
            address: context.payer.pubkey(),
            verified: true,
            share: 100,
        }]);

        let puffed_symbol = puffed_out_string(&symbol, MAX_SYMBOL_LENGTH);
        let puffed_uri = puffed_out_string(&uri, MAX_URI_LENGTH);

        test_metadata.create_v3_default(&mut context).await.unwrap();

        let updated_name = "New Name".to_string();
        let puffed_updated_name = puffed_out_string(&updated_name, MAX_NAME_LENGTH);

        test_metadata
            .update_v2(
                &mut context,
                updated_name,
                symbol,
                uri,
                creators.clone(),
                10,
                false,
                Some(Collection {
                    verified: false,
                    key: test_metadata.pubkey,
                }),
                Some(Uses {
                    use_method: UseMethod::Multiple,
                    remaining: 5,
                    total: 15,
                }),
            )
            .await
            .unwrap();

        let metadata = test_metadata.get_data(&mut context).await;
        let collection = metadata.collection.unwrap();

        assert_eq!(metadata.data.name, puffed_updated_name);
        assert_eq!(metadata.data.symbol, puffed_symbol);
        assert_eq!(metadata.data.uri, puffed_uri);
        assert_eq!(metadata.data.seller_fee_basis_points, 10);
        assert_eq!(metadata.data.creators, creators);

        assert!(!metadata.primary_sale_happened);
        assert!(!metadata.is_mutable);
        assert_eq!(metadata.mint, test_metadata.mint.pubkey());
        assert_eq!(metadata.update_authority, context.payer.pubkey());
        assert_eq!(metadata.key, Key::MetadataV1);
        assert!(collection.key == test_metadata.pubkey);
        assert!(!collection.verified);
        assert_eq!(metadata.uses.unwrap().total, 15);
    }

    #[tokio::test]
    async fn fail_update_metadata_when_collection_is_verified() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();

        test_metadata.create_v3_default(&mut context).await.unwrap();

        let new_collection_authority = Keypair::new();
        let test_collection = Metadata::new();
        test_collection
            .create_v3_default(&mut context)
            .await
            .unwrap();
        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let update_authority = context.payer.pubkey();
        let (record, _) = find_collection_authority_account(
            &test_collection.mint.pubkey(),
            &new_collection_authority.pubkey(),
        );
        let ix = mpl_token_metadata::instruction::approve_collection_authority(
            mpl_token_metadata::ID,
            record,
            new_collection_authority.pubkey(),
            update_authority,
            context.payer.pubkey(),
            test_collection.pubkey,
            test_collection.mint.pubkey(),
        );

        let tx1 = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx1).await.unwrap();

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

        let updated_name = "New Name".to_string();

        let incoming_collection = Keypair::new();

        let creators = Some(vec![Creator {
            address: context.payer.pubkey(),
            verified: true,
            share: 100,
        }]);

        let tx2 = Transaction::new_signed_with_payer(
            &[instruction::update_metadata_accounts_v2(
                ID,
                test_metadata.pubkey,
                context.payer.pubkey(),
                None,
                Some(DataV2 {
                    name: updated_name,
                    symbol: symbol.clone(),
                    uri: uri.clone(),
                    creators,
                    seller_fee_basis_points: 10,
                    collection: Some(Collection {
                        key: incoming_collection.pubkey(),
                        verified: true,
                    }),
                    uses: None,
                }),
                None,
                Some(false),
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        let result = context
            .banks_client
            .process_transaction(tx2)
            .await
            .unwrap_err();

        assert_custom_error!(
            result,
            MetadataError::CollectionCannotBeVerifiedInThisInstruction
        );
    }

    #[tokio::test]
    async fn fail_invalid_update_authority() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        let fake_update_authority = Keypair::new();

        test_metadata.create_v3_default(&mut context).await.unwrap();

        let tx = Transaction::new_signed_with_payer(
            &[instruction::update_metadata_accounts_v2(
                ID,
                test_metadata.pubkey,
                fake_update_authority.pubkey(),
                None,
                None,
                None,
                None,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, &fake_update_authority],
            context.last_blockhash,
        );

        let result = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        assert_custom_error!(result, MetadataError::UpdateAuthorityIncorrect);
    }

    #[tokio::test]
    async fn cannot_flip_primary_sale_happened_from_true_to_false() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();

        // Primary sale happened created as false by default.
        test_metadata
            .create_v3(
                &mut context,
                "Test Col".to_string(),
                "TSTCOL".to_string(),
                "uricol".to_string(),
                None,
                10,
                true,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        // Flip true.
        let tx = Transaction::new_signed_with_payer(
            &[instruction::update_metadata_accounts_v2(
                ID,
                test_metadata.pubkey,
                context.payer.pubkey(),
                None,
                None,
                Some(true),
                None,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        // Try to flip back to false; this should fail.
        let tx = Transaction::new_signed_with_payer(
            &[instruction::update_metadata_accounts_v2(
                ID,
                test_metadata.pubkey,
                context.payer.pubkey(),
                None,
                None,
                Some(false),
                None,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );
        let result = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        // We should not be able to make an immutable NFT mutable again.
        assert_custom_error!(result, MetadataError::PrimarySaleCanOnlyBeFlippedToTrue);
    }

    #[tokio::test]
    async fn cannot_flip_is_mutable_from_false_to_true() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();

        // Start with NFT immutable.
        let is_mutable = false;

        test_metadata
            .create_v3(
                &mut context,
                "Test Col".to_string(),
                "TSTCOL".to_string(),
                "uricol".to_string(),
                None,
                10,
                is_mutable,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[instruction::update_metadata_accounts_v2(
                ID,
                test_metadata.pubkey,
                context.payer.pubkey(),
                None,
                None,
                None,
                // Try to flip to be mutable.
                Some(true),
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        let result = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        // We should not be able to make an immutable NFT mutable again.
        assert_custom_error!(result, MetadataError::IsMutableCanOnlyBeFlippedToFalse);
    }

    #[tokio::test]
    async fn fail_cannot_verify_collection() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();

        test_metadata.create_v3_default(&mut context).await.unwrap();

        let test_collection = Metadata::new();
        test_collection
            .create_v3(
                &mut context,
                "Test Col".to_string(),
                "TSTCOL".to_string(),
                "uricol".to_string(),
                None,
                10,
                true,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create_v3(&mut context, Some(1))
            .await
            .unwrap();

        let creators = Some(vec![Creator {
            address: context.payer.pubkey(),
            verified: true,
            share: 100,
        }]);

        let tx = Transaction::new_signed_with_payer(
            &[instruction::update_metadata_accounts_v2(
                ID,
                test_metadata.pubkey,
                context.payer.pubkey(),
                None,
                Some(DataV2 {
                    name: "Test".to_string(),
                    symbol: "TST".to_string(),
                    uri: "uri".to_string(),
                    creators,
                    seller_fee_basis_points: 10,
                    collection: Some(Collection {
                        key: test_collection.pubkey,
                        verified: true,
                    }),
                    uses: None,
                }),
                None,
                None,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        let result = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        assert_custom_error!(
            result,
            MetadataError::CollectionCannotBeVerifiedInThisInstruction
        );
    }

    #[tokio::test]
    async fn fail_cannot_change_collection_key_when_verified() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        test_metadata.create_v3_default(&mut context).await.unwrap();

        let new_collection_authority = Keypair::new();
        let test_collection = Metadata::new();
        test_collection
            .create_v3_default(&mut context)
            .await
            .unwrap();

        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let update_authority = context.payer.pubkey();
        let (record, _) = find_collection_authority_account(
            &test_collection.mint.pubkey(),
            &new_collection_authority.pubkey(),
        );
        let ix = mpl_token_metadata::instruction::approve_collection_authority(
            mpl_token_metadata::ID,
            record,
            new_collection_authority.pubkey(),
            update_authority,
            context.payer.pubkey(),
            test_collection.pubkey,
            test_collection.mint.pubkey(),
        );

        let tx1 = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx1).await.unwrap();

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

        let creators = Some(vec![Creator {
            address: context.payer.pubkey(),
            verified: true,
            share: 100,
        }]);

        let fake_collection_pubkey = collection_master_edition_account.pubkey;
        let tx2 = Transaction::new_signed_with_payer(
            &[instruction::update_metadata_accounts_v2(
                ID,
                test_metadata.pubkey,
                context.payer.pubkey(),
                None,
                Some(DataV2 {
                    name: "Test".to_string(),
                    symbol: "TST".to_string(),
                    uri: "uri".to_string(),
                    creators,
                    seller_fee_basis_points: 10,
                    collection: Some(Collection {
                        key: fake_collection_pubkey,
                        verified: true,
                    }),
                    uses: None,
                }),
                None,
                None,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        let result = context
            .banks_client
            .process_transaction(tx2)
            .await
            .unwrap_err();

        assert_custom_error!(
            result,
            MetadataError::CollectionCannotBeVerifiedInThisInstruction
        );
    }

    #[tokio::test]
    async fn can_set_unverified_collection_data_to_none() {
        let mut context = program_test().start_with_context().await;

        let test_collection = Metadata::new();
        test_collection
            .create_v3(
                &mut context,
                "Test Col".to_string(),
                "TSTCOL".to_string(),
                "uricol".to_string(),
                None,
                10,
                true,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create_v3(&mut context, Some(1))
            .await
            .unwrap();

        let test_metadata = Metadata::new();
        test_metadata
            .create_v3(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                true,
                Some(Collection {
                    key: test_collection.pubkey,
                    verified: false,
                }),
                None,
                None,
            )
            .await
            .unwrap();

        // Setting existing, but unverified collection data to None.
        let tx = Transaction::new_signed_with_payer(
            &[instruction::update_metadata_accounts_v2(
                ID,
                test_metadata.pubkey,
                context.payer.pubkey(),
                None,
                Some(DataV2 {
                    name: "Test".to_string(),
                    symbol: "TST".to_string(),
                    uri: "uri".to_string(),
                    creators: None,
                    seller_fee_basis_points: 10,
                    collection: None,
                    uses: None,
                }),
                None,
                None,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();
        let metadata = test_metadata.get_data(&mut context).await;
        assert_eq!(metadata.collection, None);

        // Setting Collection data that's already None to None.
        let tx = Transaction::new_signed_with_payer(
            &[instruction::update_metadata_accounts_v2(
                ID,
                test_metadata.pubkey,
                context.payer.pubkey(),
                None,
                Some(DataV2 {
                    name: "Test".to_string(),
                    symbol: "TST".to_string(),
                    uri: "uri".to_string(),
                    creators: None,
                    seller_fee_basis_points: 10,
                    collection: None,
                    uses: None,
                }),
                None,
                None,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();
        let metadata = test_metadata.get_data(&mut context).await;
        assert_eq!(metadata.collection, None);
    }

    #[tokio::test]
    async fn extra_data_zeroed() {
        let mut context = program_test().start_with_context().await;

        /*
            Create a metadata account with five creators.
            Update it to have only one and ensure that all data after uses struct is zeroed out.
        */

        let creator1 = Keypair::new();
        let creator2 = Keypair::new();
        let creator3 = Keypair::new();
        let creator4 = Keypair::new();
        let creator5 = Keypair::new();

        let creators = vec![
            Creator {
                address: creator1.pubkey(),
                verified: false,
                share: 20,
            },
            Creator {
                address: creator2.pubkey(),
                verified: false,
                share: 20,
            },
            Creator {
                address: creator3.pubkey(),
                verified: false,
                share: 20,
            },
            Creator {
                address: creator4.pubkey(),
                verified: false,
                share: 20,
            },
            Creator {
                address: creator5.pubkey(),
                verified: false,
                share: 20,
            },
        ];

        let test_metadata = Metadata::new();
        test_metadata
            .create_v3(
                &mut context,
                "Test Col".to_string(),
                "TSTCOL".to_string(),
                "uricol".to_string(),
                Some(creators),
                10,
                true,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        test_metadata
            .update_v2(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                Some(vec![Creator {
                    address: creator1.pubkey(),
                    verified: false,
                    share: 100,
                }]),
                10,
                true,
                None,
                None,
            )
            .await
            .unwrap();

        let data = get_account(&mut context, &test_metadata.pubkey).await.data;

        let padding_index = 1 + 32 + 32 + 36 + 14 + 204 + 7 + 34 + 1 + 1 + 2 + 2 + 1 + 1;
        let zeros_len = data.len() - padding_index - 1; // Fee flag at end
        let zeros = vec![0u8; zeros_len];
        assert_eq!(data[padding_index..data.len() - 1], zeros[..]);
    }

    #[tokio::test]
    async fn fail_invalid_use_method() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();

        test_metadata
            .create_v3(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                true,
                None,
                Some(Uses {
                    use_method: UseMethod::Single,
                    remaining: 1,
                    total: 1,
                }),
                None,
            )
            .await
            .unwrap();

        let tx = Transaction::new_signed_with_payer(
            &[instruction::update_metadata_accounts_v2(
                ID,
                test_metadata.pubkey,
                context.payer.pubkey(),
                None,
                Some(DataV2 {
                    name: "Test".to_string(),
                    symbol: "TST".to_string(),
                    uri: "uri".to_string(),
                    creators: None,
                    seller_fee_basis_points: 10,
                    collection: None,
                    uses: Some(Uses {
                        use_method: UseMethod::Multiple,
                        remaining: 1,
                        total: 1,
                    }),
                }),
                None,
                None,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        let result = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        assert_custom_error!(result, MetadataError::InvalidUseMethod);
    }

    #[tokio::test]
    async fn fail_cannot_unverify_another_creator_by_changing_array() {
        let mut context = program_test().start_with_context().await;

        // Create metadata with one verified creator.
        let test_metadata = Metadata::new();
        test_metadata.create_v3_default(&mut context).await.unwrap();

        // Update authority.
        let new_update_authority = Keypair::new();
        let tx = Transaction::new_signed_with_payer(
            &[instruction::update_metadata_accounts_v2(
                ID,
                test_metadata.pubkey,
                context.payer.pubkey(),
                Some(new_update_authority.pubkey()),
                None,
                None,
                None,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        // Try to update metadata with a different verified creator.
        let new_creators = vec![
            Creator {
                address: context.payer.pubkey(),
                verified: false,
                share: 50,
            },
            Creator {
                address: new_update_authority.pubkey(),
                verified: true,
                share: 50,
            },
        ];

        let tx = Transaction::new_signed_with_payer(
            &[instruction::update_metadata_accounts_v2(
                ID,
                test_metadata.pubkey,
                new_update_authority.pubkey(),
                None,
                Some(DataV2 {
                    name: "Test".to_string(),
                    symbol: "TST".to_string(),
                    uri: "uri".to_string(),
                    creators: Some(new_creators),
                    seller_fee_basis_points: 10,
                    collection: None,
                    uses: None,
                }),
                None,
                None,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, &new_update_authority],
            context.last_blockhash,
        );

        let result = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        assert_custom_error!(result, MetadataError::CannotUnverifyAnotherCreator);
    }
}

#[tokio::test]
async fn fail_cannot_unverify_another_creator_by_removing_from_array() {
    let mut context = program_test().start_with_context().await;

    // Create metadata with one verified creator.
    let test_metadata = Metadata::new();
    test_metadata.create_v3_default(&mut context).await.unwrap();

    // Update authority.
    let new_update_authority = Keypair::new();
    let tx = Transaction::new_signed_with_payer(
        &[instruction::update_metadata_accounts_v2(
            ID,
            test_metadata.pubkey,
            context.payer.pubkey(),
            Some(new_update_authority.pubkey()),
            None,
            None,
            None,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Try to update metadata with a different verified creator.
    let new_creators = vec![Creator {
        address: new_update_authority.pubkey(),
        verified: true,
        share: 100,
    }];

    let tx = Transaction::new_signed_with_payer(
        &[instruction::update_metadata_accounts_v2(
            ID,
            test_metadata.pubkey,
            new_update_authority.pubkey(),
            None,
            Some(DataV2 {
                name: "Test".to_string(),
                symbol: "TST".to_string(),
                uri: "uri".to_string(),
                creators: Some(new_creators),
                seller_fee_basis_points: 10,
                collection: None,
                uses: None,
            }),
            None,
            None,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &new_update_authority],
        context.last_blockhash,
    );

    let result = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_error!(result, MetadataError::CannotUnverifyAnotherCreator);
}

#[tokio::test]
async fn fail_cannot_unverify_creators_by_setting_to_none() {
    let mut context = program_test().start_with_context().await;

    // Create metadata with one verified creator.
    let test_metadata = Metadata::new();
    test_metadata.create_v3_default(&mut context).await.unwrap();

    // Update authority.
    let new_update_authority = Keypair::new();
    let tx = Transaction::new_signed_with_payer(
        &[instruction::update_metadata_accounts_v2(
            ID,
            test_metadata.pubkey,
            context.payer.pubkey(),
            Some(new_update_authority.pubkey()),
            None,
            None,
            None,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Try to update metadata by setting creators to None.
    let tx = Transaction::new_signed_with_payer(
        &[instruction::update_metadata_accounts_v2(
            ID,
            test_metadata.pubkey,
            new_update_authority.pubkey(),
            None,
            Some(DataV2 {
                name: "Test".to_string(),
                symbol: "TST".to_string(),
                uri: "uri".to_string(),
                creators: None,
                seller_fee_basis_points: 10,
                collection: None,
                uses: None,
            }),
            None,
            None,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &new_update_authority],
        context.last_blockhash,
    );

    let result = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_error!(result, MetadataError::CannotRemoveVerifiedCreator);
}
