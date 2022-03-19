#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::{
    error::MetadataError,
    id, instruction,
    state::{Collection, Key, UseMethod, Uses, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH},
    utils::puffed_out_string,
};
use num_traits::FromPrimitive;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};
use utils::*;

mod reset_v2_metadata {
    use super::*;

    #[tokio::test]
    async fn success() {
        // ARRANGE
        let mut context = program_test().start_with_context().await;

        let update_authority = clone_keypair(&context.payer);

        // Create a test collection to use in our metadata.
        let test_collection = Metadata::new();
        test_collection
            .create_v2(
                &mut context,
                "Test Collection".to_string(),
                "".to_string(),
                "uri".to_string(),
                None,
                0,
                false,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        // Create metadata to test clearing v2 data.
        let test_metadata = Metadata::new();
        let name = "Test".to_string();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();

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
                None,
                Some(Collection {
                    key: test_collection.mint.pubkey(),
                    verified: false,
                }),
                uses.to_owned(),
            )
            .await
            .unwrap();

        // ACT
        let metadata = test_metadata.get_data(&mut context).await;
        test_metadata
            .reset_v2_metadata(&mut context, update_authority)
            .await
            .unwrap();
        let post_metadata = test_metadata.get_data(&mut context).await;

        // ASSERT

        // Metadata V1 values are correct.
        assert_eq!(metadata.key, Key::MetadataV1);
        assert_eq!(metadata.update_authority, context.payer.pubkey());
        assert_eq!(metadata.mint, test_metadata.mint.pubkey());
        assert_eq!(metadata.data.name, puffed_name);
        assert_eq!(metadata.data.symbol, puffed_symbol);
        assert_eq!(metadata.data.uri, puffed_uri);
        assert_eq!(metadata.data.seller_fee_basis_points, 10);
        assert_eq!(metadata.data.creators, None);
        assert_eq!(metadata.primary_sale_happened, false);
        assert_eq!(metadata.is_mutable, false);

        // Metadata has correct collection values.
        assert_eq!(
            metadata.collection.to_owned().unwrap().key,
            test_collection.mint.pubkey()
        );
        assert_eq!(metadata.collection.unwrap().verified, false);

        // Metadata has correct use values.
        assert_eq!(metadata.uses, uses.to_owned());

        // Post metadata has new values cleared but other data is untouched.
        assert_eq!(post_metadata.key, Key::MetadataV1);
        assert_eq!(post_metadata.update_authority, context.payer.pubkey());
        assert_eq!(post_metadata.mint, test_metadata.mint.pubkey());
        assert_eq!(post_metadata.data.name, puffed_name);
        assert_eq!(post_metadata.data.symbol, puffed_symbol);
        assert_eq!(post_metadata.data.uri, puffed_uri);
        assert_eq!(post_metadata.data.seller_fee_basis_points, 10);
        assert_eq!(post_metadata.data.creators, None);
        assert_eq!(post_metadata.primary_sale_happened, false);
        assert_eq!(post_metadata.is_mutable, false);

        assert_eq!(post_metadata.token_standard, None);
        assert_eq!(post_metadata.collection, None);
        assert_eq!(post_metadata.uses, None);
    }

    #[tokio::test]
    async fn incorrect_update_authority_fails() {
        // ARRANGE
        let mut context = program_test().start_with_context().await;

        // Create a test collection to use in our metadata.
        let test_collection = Metadata::new();
        test_collection
            .create_v2(
                &mut context,
                "Test Collection".to_string(),
                "".to_string(),
                "uri".to_string(),
                None,
                0,
                false,
                None,
                None,
                None,
            )
            .await
            .expect("Failed to create test collection.");
        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let collection = Collection {
            key: test_collection.mint.pubkey(),
            verified: false,
        };

        // Create metadata to test clearing v2 data.
        let test_metadata = Metadata::new();
        let name = "Test".to_string();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();

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
                None,
                Some(collection.clone()),
                uses.to_owned(),
            )
            .await
            .expect("Failed to create test metadata.");

        let incorrect_update_authority = Keypair::new();

        // ACT

        // Fund invalid update authority.
        airdrop(
            &mut context,
            &incorrect_update_authority.pubkey(),
            10000000000,
        )
        .await
        .expect("Airdrop failed");

        let metadata = test_metadata.get_data(&mut context).await;

        // Try to reset v2 metadata using incorrect update authority.
        let tx = Transaction::new_signed_with_payer(
            &[instruction::reset_v2_metadata(
                id(),
                test_metadata.pubkey,
                incorrect_update_authority.pubkey(),
            )],
            Some(&incorrect_update_authority.pubkey()),
            &[&incorrect_update_authority],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        let post_metadata = test_metadata.get_data(&mut context).await;

        // ASSERT
        assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);

        // Post metadata should continue to have old values.
        assert_eq!(post_metadata.token_standard, metadata.token_standard);
        assert_eq!(post_metadata.collection, Some(collection));
        assert_eq!(post_metadata.uses, uses);
    }

    #[tokio::test]
    #[should_panic(expected = "Transaction::sign failed with error KeypairPubkeyMismatch")]
    async fn update_authority_not_a_signer_fails() {
        // ARRANGE
        let mut context = program_test().start_with_context().await;

        let payer = Keypair::new();
        let fake_signer = Keypair::new();

        // Create a test collection to use in our metadata.
        let test_collection = Metadata::new();
        test_collection
            .create_v2(
                &mut context,
                "Test Collection".to_string(),
                "".to_string(),
                "uri".to_string(),
                None,
                0,
                false,
                None,
                None,
                None,
            )
            .await
            .expect("Failed to create test collection.");
        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let collection = Collection {
            key: test_collection.mint.pubkey(),
            verified: false,
        };

        // Create metadata to test clearing v2 data.
        let test_metadata = Metadata::new();
        let name = "Test".to_string();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();

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
                None,
                Some(collection.clone()),
                uses.to_owned(),
            )
            .await
            .expect("Failed to create test metadata.");

        let update_authority_pubkey = context.payer.pubkey();

        // ACT

        let metadata = test_metadata.get_data(&mut context).await;

        // Fund payer account
        airdrop(&mut context, &payer.pubkey(), 10000000000)
            .await
            .expect("Airdrop failed");

        // Try to reset v2 metadata using the correct update authority but which is not a signer.
        let tx = Transaction::new_signed_with_payer(
            &[instruction::reset_v2_metadata(
                id(),
                test_metadata.pubkey,
                update_authority_pubkey,
            )],
            Some(&payer.pubkey()),
            &[&payer, &fake_signer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let post_metadata = test_metadata.get_data(&mut context).await;

        // ASSERT
        // Post metadata should continue to have old values.
        assert_eq!(post_metadata.token_standard, metadata.token_standard);
        assert_eq!(post_metadata.collection, Some(collection));
        assert_eq!(post_metadata.uses, uses);
    }
}
