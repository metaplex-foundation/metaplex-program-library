#![cfg(feature = "test-bpf")]
pub mod utils;

use borsh::BorshDeserialize;
use mpl_token_metadata::{
    error::MetadataError,
    instruction::set_collection_status,
    state::Metadata as ProgramMetadata,
    state::{CollectionDetails, CollectionStatus},
    ID as PROGRAM_ID,
};
use num_traits::FromPrimitive;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};
use utils::*;

mod set_collection_status {
    use super::*;

    #[tokio::test]
    async fn successfully_update_status() {
        let mut context = program_test().start_with_context().await;

        // Create a Collection Parent NFT with the CollectionDetails struct populated
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
                true, // is collection parent
            )
            .await
            .unwrap();
        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let current_status = CollectionStatus::None;
        let new_status = CollectionStatus::Announced;

        let md_account = context
            .banks_client
            .get_account(collection_parent_nft.pubkey)
            .await
            .unwrap()
            .unwrap();

        let metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();
        let retrieved_status = if let CollectionDetails::CollectionDetailsV1 { status, size: _ } =
            metadata.collection_details
        {
            status
        } else {
            panic!("Expected CollectionDetails::CollectionDetailsV1");
        };

        assert_eq!(retrieved_status, current_status);

        let ix = set_collection_status(
            PROGRAM_ID,
            collection_parent_nft.pubkey,
            context.payer.pubkey(),
            new_status,
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
        let retrieved_status = if let CollectionDetails::CollectionDetailsV1 { status, size: _ } =
            metadata.collection_details
        {
            status
        } else {
            panic!("Expected CollectionDetails::CollectionDetailsV1");
        };

        assert_eq!(retrieved_status, new_status);
    }

    #[tokio::test]
    async fn invalid_update_authority() {
        let mut context = program_test().start_with_context().await;

        let invalid_update_authority = Keypair::new();

        airdrop(
            &mut context,
            &invalid_update_authority.pubkey(),
            1_000_000_000,
        )
        .await
        .unwrap();

        // Create a Collection Parent NFT with the CollectionDetails struct populated
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
                true, // is collection parent
            )
            .await
            .unwrap();
        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let current_status = CollectionStatus::None;
        let new_status = CollectionStatus::Announced;

        let md_account = context
            .banks_client
            .get_account(collection_parent_nft.pubkey)
            .await
            .unwrap()
            .unwrap();

        let metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();
        let retrieved_status = if let CollectionDetails::CollectionDetailsV1 { status, size: _ } =
            metadata.collection_details
        {
            status
        } else {
            panic!("Expected CollectionDetails::CollectionDetailsV1");
        };

        assert_eq!(retrieved_status, current_status);

        let ix = set_collection_status(
            PROGRAM_ID,
            collection_parent_nft.pubkey,
            invalid_update_authority.pubkey(),
            new_status,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&invalid_update_authority.pubkey()),
            &[&invalid_update_authority],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);
    }

    #[tokio::test]
    async fn incorrect_update_authority() {
        let mut context = program_test().start_with_context().await;

        // This key will pay for the transaction, but is not the update authority.
        let payer = Keypair::new();
        airdrop(&mut context, &payer.pubkey(), 1_000_000_000)
            .await
            .unwrap();

        // This keypair is used to make the correct number of signers and get around that pre-flight error.
        let additional_signer = Keypair::new();

        // Create a Collection Parent NFT with the CollectionDetails struct populated
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
                true, // is collection parent
            )
            .await
            .unwrap();
        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let current_status = CollectionStatus::None;
        let new_status = CollectionStatus::Announced;

        let md_account = context
            .banks_client
            .get_account(collection_parent_nft.pubkey)
            .await
            .unwrap()
            .unwrap();

        let metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();
        let retrieved_status = if let CollectionDetails::CollectionDetailsV1 { status, size: _ } =
            metadata.collection_details
        {
            status
        } else {
            panic!("Expected CollectionDetails::CollectionDetailsV1");
        };

        assert_eq!(retrieved_status, current_status);

        let ix = set_collection_status(
            PROGRAM_ID,
            collection_parent_nft.pubkey,
            // context.payer.pubkey(),
            additional_signer.pubkey(),
            new_status,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &additional_signer],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);
    }

    #[tokio::test]
    async fn invalid_metadata_account() {
        let mut context = program_test().start_with_context().await;

        // Submit a tx with a metadata account not owned by the token-metadata program.
        // This should fail with IncorrectOwner error.

        let fake_metadata = Keypair::new();

        let new_status = CollectionStatus::Announced;

        let ix = set_collection_status(
            PROGRAM_ID,
            fake_metadata.pubkey(),
            context.payer.pubkey(),
            new_status,
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
}
