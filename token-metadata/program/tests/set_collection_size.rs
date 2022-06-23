#![cfg(feature = "test-bpf")]
pub mod utils;

use borsh::BorshDeserialize;
use mpl_token_metadata::{
    error::MetadataError,
    instruction::{approve_collection_authority, set_collection_size},
    state::{CollectionDetails, Metadata as ProgramMetadata},
    ID as PROGRAM_ID,
};
use num_traits::FromPrimitive;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction, TransactionError},
};
use utils::*;

mod set_collection_size {

    use mpl_token_metadata::pda::find_collection_authority_account;

    use super::*;

    #[tokio::test]
    async fn collection_authority_successfully_updates_size() {
        let mut context = program_test().start_with_context().await;

        // Create a Collection Parent NFT with the CollectionDetails set to None
        let collection_parent_nft = Metadata::new();
        collection_parent_nft
            .create_v3(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let size = 1123;

        let ix = set_collection_size(
            PROGRAM_ID,
            collection_parent_nft.pubkey,
            context.payer.pubkey(),
            collection_parent_nft.mint.pubkey(),
            None,
            size,
        );

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();
        let md_account = context
            .banks_client
            .get_account(collection_parent_nft.pubkey)
            .await
            .unwrap()
            .unwrap();

        let metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();
        let retrieved_size = if let Some(details) = metadata.collection_details {
            match details {
                CollectionDetails::V1 { size } => size,
            }
        } else {
            panic!("Expected CollectionDetails::V1");
        };

        assert_eq!(retrieved_size, size);
    }

    #[tokio::test]
    async fn delegate_authority_successfully_updates_size() {
        let mut context = program_test().start_with_context().await;

        // Create a Collection Parent NFT with the CollectionDetails set to None
        let collection_parent_nft = Metadata::new();
        collection_parent_nft
            .create_v3(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        // NFT is created with context payer as the update authority so we need to update this so we don't automatically
        // get the update authority to sign the transaction.
        let new_update_authority = Keypair::new();

        collection_parent_nft
            .change_update_authority(&mut context, new_update_authority.pubkey())
            .await
            .unwrap();

        // Approve a delegate collection authority.
        let delegate = Keypair::new();

        // Derive collection authority record.
        let (collection_authority_record, _) = find_collection_authority_account(
            &collection_parent_nft.mint.pubkey(),
            &delegate.pubkey(),
        );

        let ix = approve_collection_authority(
            PROGRAM_ID,
            collection_authority_record,
            delegate.pubkey(),
            new_update_authority.pubkey(),
            context.payer.pubkey(),
            collection_parent_nft.pubkey,
            collection_parent_nft.mint.pubkey(),
        );

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &new_update_authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let size = 1123;

        let ix = set_collection_size(
            PROGRAM_ID,
            collection_parent_nft.pubkey,
            delegate.pubkey(),
            collection_parent_nft.mint.pubkey(),
            Some(collection_authority_record),
            size,
        );

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &delegate],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();
        let md_account = context
            .banks_client
            .get_account(collection_parent_nft.pubkey)
            .await
            .unwrap()
            .unwrap();

        let metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();
        let retrieved_size = if let Some(details) = metadata.collection_details {
            match details {
                CollectionDetails::V1 { size } => size,
            }
        } else {
            panic!("Expected CollectionDetails::V1");
        };

        assert_eq!(retrieved_size, size);
    }

    #[tokio::test]
    async fn invalid_metadata_account() {
        // Submit a tx with a metadata account not owned by the token-metadata program.
        // This should fail with IncorrectOwner error.
        let mut context = program_test().start_with_context().await;

        // Create a Collection Parent NFT with the CollectionDetails set to None
        let collection_parent_nft = Metadata::new();
        collection_parent_nft
            .create_v3(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let new_size = 1123;

        let fake_metadata = Keypair::new();

        let ix = set_collection_size(
            PROGRAM_ID,
            fake_metadata.pubkey(),
            context.payer.pubkey(),
            collection_parent_nft.mint.pubkey(),
            None,
            new_size,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::IncorrectOwner);
    }

    #[tokio::test]
    async fn invalid_update_authority_fails() {
        let mut context = program_test().start_with_context().await;

        // Create a Collection Parent NFT with the CollectionDetails set to None
        let collection_parent_nft = Metadata::new();
        collection_parent_nft
            .create_v3(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        // NFT is created with context payer as the update authority so we need to update this so we don't automatically
        // get the update authority to sign the transaction.
        let new_update_authority = Keypair::new();

        collection_parent_nft
            .change_update_authority(&mut context, new_update_authority.pubkey())
            .await
            .unwrap();

        let invalid_update_authorty = Keypair::new();

        let size = 1123;

        let ix = set_collection_size(
            PROGRAM_ID,
            collection_parent_nft.pubkey,
            invalid_update_authorty.pubkey(),
            collection_parent_nft.mint.pubkey(),
            None,
            size,
        );

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &invalid_update_authorty],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidCollectionUpdateAuthority);
    }

    #[tokio::test]
    async fn fail_to_update_sized_collection() {
        let mut context = program_test().start_with_context().await;

        // Create a Collection Parent NFT with the CollectionDetails populated (sized)
        let collection_parent_nft = Metadata::new();
        collection_parent_nft
            .create_v3(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                None,
                None,
                DEFAULT_COLLECTION_DETAILS,
            )
            .await
            .unwrap();
        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let size = 1123;

        let ix = set_collection_size(
            PROGRAM_ID,
            collection_parent_nft.pubkey,
            context.payer.pubkey(),
            collection_parent_nft.mint.pubkey(),
            None,
            size,
        );

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        // This should fail with SizedCollection error.
        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();
        assert_custom_error!(err, MetadataError::SizedCollection);

        let md_account = context
            .banks_client
            .get_account(collection_parent_nft.pubkey)
            .await
            .unwrap()
            .unwrap();

        let metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();
        let retrieved_size = if let Some(details) = metadata.collection_details {
            match details {
                CollectionDetails::V1 { size } => size,
            }
        } else {
            panic!("Expected CollectionDetails::V1");
        };

        // The size should not have changed.
        assert_eq!(retrieved_size, 0);
    }

    #[tokio::test]
    async fn can_only_set_size_once() {
        let mut context = program_test().start_with_context().await;

        // Create a Collection Parent NFT with the CollectionDetails set to None (unsized)
        let collection_parent_nft = Metadata::new();
        collection_parent_nft
            .create_v3(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let size = 1123;

        let ix = set_collection_size(
            PROGRAM_ID,
            collection_parent_nft.pubkey,
            context.payer.pubkey(),
            collection_parent_nft.mint.pubkey(),
            None,
            size,
        );

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let md_account = context
            .banks_client
            .get_account(collection_parent_nft.pubkey)
            .await
            .unwrap()
            .unwrap();

        let metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();
        let retrieved_size = if let Some(details) = metadata.collection_details {
            match details {
                CollectionDetails::V1 { size } => size,
            }
        } else {
            panic!("Expected CollectionDetails::V1");
        };

        // First update should work.
        assert_eq!(retrieved_size, size);

        let new_size = 3211;

        let ix = set_collection_size(
            PROGRAM_ID,
            collection_parent_nft.pubkey,
            context.payer.pubkey(),
            collection_parent_nft.mint.pubkey(),
            None,
            new_size,
        );

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        // This should fail with SizedCollection error.
        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();
        assert_custom_error!(err, MetadataError::SizedCollection);
    }
}
