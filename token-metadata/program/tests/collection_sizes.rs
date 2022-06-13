#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::state::Metadata as ProgramMetadata;
use num_traits::FromPrimitive;
use solana_program_test::*;
use solana_sdk::{instruction::InstructionError, transaction::TransactionError};
use utils::*;

mod unsized_collection_handlers {

    use mpl_token_metadata::{error::MetadataError, state::Collection};
    use solana_sdk::{signature::Keypair, signer::Signer};

    use super::*;

    #[tokio::test]
    async fn old_verify_cant_change_size() {
        let mut context = program_test().start_with_context().await;

        // Create a Collection Item NFT belonging to the collection parent
        // Try to verify the collection item NFT w/ old verify handler

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
                DEFAULT_COLLECTION_DETAILS, // Collection Parent
            )
            .await
            .unwrap();
        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let collection = Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        };

        let collection_item_nft = Metadata::new();
        collection_item_nft
            .create_v3(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                Some(collection),
                None,
                None, // is not collection parent
            )
            .await
            .unwrap();
        let item_master_edition_account = MasterEditionV2::new(&collection_item_nft);
        item_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Try to verify the item with the old handler.
        let err = collection_item_nft
            .verify_collection(
                &mut context,
                collection_parent_nft.pubkey,
                &payer,
                collection_parent_nft.mint.pubkey(),
                parent_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::SizedCollection);
    }

    #[tokio::test]
    async fn old_unverify_cant_change_size() {
        let mut context = program_test().start_with_context().await;

        // Create a Collection Parent NFT with the CollectionDetails field populated
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
                DEFAULT_COLLECTION_DETAILS, // is collection parent
            )
            .await
            .unwrap();
        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let collection = Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        };

        let collection_item_nft = Metadata::new();
        collection_item_nft
            .create_v3(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                Some(collection),
                None,
                None, // is not collection parent
            )
            .await
            .unwrap();
        let item_master_edition_account = MasterEditionV2::new(&collection_item_nft);
        item_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Verify the item so we can try to unverify it.
        collection_item_nft
            .verify_sized_collection_item(
                &mut context,
                collection_parent_nft.pubkey,
                &payer,
                collection_parent_nft.mint.pubkey(),
                parent_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap();

        // Try to unverify. This should fail as we can't unverify and set size of a sized collection
        // with this handler.
        let err = collection_item_nft
            .unverify_collection(
                &mut context,
                collection_parent_nft.pubkey,
                &payer,
                collection_parent_nft.mint.pubkey(),
                parent_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::SizedCollection);
    }

    #[tokio::test]
    async fn old_set_and_verify_cant_change_size() {
        let mut context = program_test().start_with_context().await;

        // Create a Collection Parent NFT with the CollectionDetails field populated
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
                DEFAULT_COLLECTION_DETAILS, // is collection parent
            )
            .await
            .unwrap();
        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        let collection_item_nft = Metadata::new();
        collection_item_nft
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
                None, // is not collection parent
            )
            .await
            .unwrap();
        let item_master_edition_account = MasterEditionV2::new(&collection_item_nft);
        item_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let err = collection_item_nft
            .set_and_verify_collection(
                &mut context,
                collection_parent_nft.pubkey,
                &payer,
                payer.pubkey(),
                collection_parent_nft.mint.pubkey(),
                parent_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::SizedCollection);
    }
}

mod sized_collection_handlers {
    use mpl_token_metadata::{error::MetadataError, state::Collection};
    use solana_sdk::{signature::Keypair, signer::Signer};

    use super::*;
    #[tokio::test]
    async fn new_verify_cant_verify_unsized() {
        let mut context = program_test().start_with_context().await;

        // Create a Collection Parent NFT without the CollectionDetails struct populated
        let collection_parent_nft = Metadata::new();
        collection_parent_nft
            .create_v2(
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
            )
            .await
            .unwrap();
        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let collection = Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        };

        let collection_item_nft = Metadata::new();
        collection_item_nft
            .create_v2(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                Some(collection),
                None,
            )
            .await
            .unwrap();
        let item_master_edition_account = MasterEditionV2::new(&collection_item_nft);
        item_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Try to verify the item with the new handler.
        let err = collection_item_nft
            .verify_sized_collection_item(
                &mut context,
                collection_parent_nft.pubkey,
                &payer,
                collection_parent_nft.mint.pubkey(),
                parent_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UnsizedCollection);
    }

    #[tokio::test]
    async fn new_unverify_cant_unverify_unsized() {
        let mut context = program_test().start_with_context().await;

        // Create a Collection Parent NFT without the CollectionDetails field populated
        let collection_parent_nft = Metadata::new();
        collection_parent_nft
            .create_v2(
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
            )
            .await
            .unwrap();
        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let collection = Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        };

        let collection_item_nft = Metadata::new();
        collection_item_nft
            .create_v2(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                Some(collection),
                None,
            )
            .await
            .unwrap();
        let item_master_edition_account = MasterEditionV2::new(&collection_item_nft);
        item_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Verify the item so we can try to unverify it.
        collection_item_nft
            .verify_collection(
                &mut context,
                collection_parent_nft.pubkey,
                &payer,
                collection_parent_nft.mint.pubkey(),
                parent_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap();

        let err = collection_item_nft
            .unverify_sized_collection_item(
                &mut context,
                collection_parent_nft.pubkey,
                &payer,
                collection_parent_nft.mint.pubkey(),
                parent_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UnsizedCollection);
    }

    #[tokio::test]
    async fn new_set_and_verify_cant_change_unsized() {
        let mut context = program_test().start_with_context().await;

        // Create a Collection Parent NFT without the CollectionDetails field populated
        let collection_parent_nft = Metadata::new();
        collection_parent_nft
            .create_v2(
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
            )
            .await
            .unwrap();
        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        let collection_item_nft = Metadata::new();
        collection_item_nft
            .create_v2(
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
            )
            .await
            .unwrap();
        let item_master_edition_account = MasterEditionV2::new(&collection_item_nft);
        item_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let err = collection_item_nft
            .set_and_verify_sized_collection_item(
                &mut context,
                collection_parent_nft.pubkey,
                &payer,
                payer.pubkey(),
                collection_parent_nft.mint.pubkey(),
                parent_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UnsizedCollection);
    }
}

mod size_tracking {
    use borsh::BorshDeserialize;
    use mpl_token_metadata::state::{Collection, CollectionDetails};
    use solana_sdk::{signature::Keypair, signer::Signer};

    use super::*;
    #[tokio::test]
    async fn increment_and_decrement_successfully() {
        let mut context = program_test().start_with_context().await;

        // Create a Collection Parent NFT without the CollectionDetails struct populated
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
                DEFAULT_COLLECTION_DETAILS, // Collection Parent
            )
            .await
            .unwrap();
        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let collection = Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        };

        let collection_item_nft = Metadata::new();
        collection_item_nft
            .create_v3(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                Some(collection),
                None,
                None, // Collection Item
            )
            .await
            .unwrap();
        let item_master_edition_account = MasterEditionV2::new(&collection_item_nft);
        item_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        let parent_nft_account = get_account(&mut context, &collection_parent_nft.pubkey).await;
        let parent_metadata =
            ProgramMetadata::deserialize(&mut parent_nft_account.data.as_slice()).unwrap();

        if let Some(details) = parent_metadata.collection_details {
            match details {
                CollectionDetails::V1 { size } => {
                    assert_eq!(size, 0);
                }
            }
        } else {
            panic!("CollectionDetails is not populated!");
        }

        // Verifying increments the size.
        collection_item_nft
            .verify_sized_collection_item(
                &mut context,
                collection_parent_nft.pubkey,
                &payer,
                collection_parent_nft.mint.pubkey(),
                parent_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap();

        let parent_nft_account = get_account(&mut context, &collection_parent_nft.pubkey).await;
        let parent_metadata =
            ProgramMetadata::deserialize(&mut parent_nft_account.data.as_slice()).unwrap();

        if let Some(details) = parent_metadata.collection_details {
            match details {
                CollectionDetails::V1 { size } => assert_eq!(size, 1),
            }
        } else {
            panic!("CollectionDetails is not populated!");
        }

        // Unverifying decrements the size.
        collection_item_nft
            .unverify_sized_collection_item(
                &mut context,
                collection_parent_nft.pubkey,
                &payer,
                collection_parent_nft.mint.pubkey(),
                parent_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap();

        let parent_nft_account = get_account(&mut context, &collection_parent_nft.pubkey).await;
        let parent_metadata =
            ProgramMetadata::deserialize(&mut parent_nft_account.data.as_slice()).unwrap();

        if let Some(details) = parent_metadata.collection_details {
            match details {
                CollectionDetails::V1 { size } => assert_eq!(size, 0),
            }
        } else {
            panic!("CollectionDetails is not populated!");
        }

        // Set-and-verify increments the size.
        collection_item_nft
            .set_and_verify_sized_collection_item(
                &mut context,
                collection_parent_nft.pubkey,
                &payer,
                payer.pubkey(),
                collection_parent_nft.mint.pubkey(),
                parent_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap();

        let parent_nft_account = get_account(&mut context, &collection_parent_nft.pubkey).await;
        let parent_metadata =
            ProgramMetadata::deserialize(&mut parent_nft_account.data.as_slice()).unwrap();

        if let Some(details) = parent_metadata.collection_details {
            match details {
                CollectionDetails::V1 { size } => assert_eq!(size, 1),
            }
        } else {
            panic!("CollectionDetails is not populated!");
        }

        // Burning decrements the size.
        burn(
            &mut context,
            collection_item_nft.pubkey,
            &payer,
            collection_item_nft.mint.pubkey(),
            collection_item_nft.token.pubkey(),
            item_master_edition_account.pubkey,
            Some(collection_parent_nft.pubkey),
        )
        .await
        .unwrap();

        let parent_nft_account = get_account(&mut context, &collection_parent_nft.pubkey).await;
        let parent_metadata =
            ProgramMetadata::deserialize(&mut parent_nft_account.data.as_slice()).unwrap();

        if let Some(details) = parent_metadata.collection_details {
            match details {
                CollectionDetails::V1 { size } => assert_eq!(size, 0),
            }
        } else {
            panic!("CollectionDetails is not populated!");
        }
    }
}
