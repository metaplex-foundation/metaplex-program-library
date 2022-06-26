#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::state::Metadata as ProgramMetadata;
use num_traits::FromPrimitive;
use solana_program_test::*;
use solana_sdk::{instruction::InstructionError, signer::Signer, transaction::TransactionError};
use utils::*;
mod burn_nft {

    use borsh::BorshDeserialize;
    use mpl_token_metadata::{
        error::MetadataError,
        state::{Collection, CollectionDetails},
    };
    use solana_sdk::signature::Keypair;

    use super::*;
    #[tokio::test]
    async fn successfully_burn_master_edition_nft() {
        let mut context = program_test().start_with_context().await;

        let test_metadata = Metadata::new();
        test_metadata
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

        let master_edition = MasterEditionV2::new(&test_metadata);
        master_edition
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        // Metadata, Master Edition and token account exist.
        let md_account = context
            .banks_client
            .get_account(test_metadata.pubkey)
            .await
            .unwrap();
        let token_account = context
            .banks_client
            .get_account(test_metadata.token.pubkey())
            .await
            .unwrap();
        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap();

        assert!(md_account.is_some());
        assert!(token_account.is_some());
        assert!(master_edition_account.is_some());

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        burn(
            &mut context,
            test_metadata.pubkey,
            &payer,
            test_metadata.mint.pubkey(),
            test_metadata.token.pubkey(),
            master_edition.pubkey,
            None,
        )
        .await
        .unwrap();

        // Metadata, Master Edition and token account are burned.
        let md_account = context
            .banks_client
            .get_account(test_metadata.pubkey)
            .await
            .unwrap();
        let token_account = context
            .banks_client
            .get_account(test_metadata.token.pubkey())
            .await
            .unwrap();
        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap();

        assert!(md_account.is_none());
        assert!(token_account.is_none());
        assert!(master_edition_account.is_none());
    }

    #[tokio::test]
    async fn fail_to_burn_print_edition() {
        let mut context = program_test().start_with_context().await;

        let original_nft = Metadata::new();
        original_nft
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

        let master_edition = MasterEditionV2::new(&original_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        // Metadata, Print Edition and token account exist.
        let md_account = context
            .banks_client
            .get_account(print_edition.new_metadata_pubkey)
            .await
            .unwrap();
        let token_account = context
            .banks_client
            .get_account(print_edition.token.pubkey())
            .await
            .unwrap();
        let print_edition_account = context
            .banks_client
            .get_account(print_edition.new_edition_pubkey)
            .await
            .unwrap();

        assert!(md_account.is_some());
        assert!(token_account.is_some());
        assert!(print_edition_account.is_some());

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        let error = burn(
            &mut context,
            print_edition.new_metadata_pubkey,
            &payer,
            print_edition.mint.pubkey(),
            print_edition.token.pubkey(),
            print_edition.new_edition_pubkey,
            None,
        )
        .await
        .unwrap_err();

        assert_custom_error!(error, MetadataError::NotAMasterEdition);
    }

    #[tokio::test]
    async fn fail_to_burn_master_edition_with_existing_prints() {
        let mut context = program_test().start_with_context().await;

        let original_nft = Metadata::new();
        original_nft
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

        let master_edition = MasterEditionV2::new(&original_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        // Metadata, Print Edition and token account exist.
        let md_account = context
            .banks_client
            .get_account(print_edition.new_metadata_pubkey)
            .await
            .unwrap();
        let token_account = context
            .banks_client
            .get_account(print_edition.token.pubkey())
            .await
            .unwrap();
        let print_edition_account = context
            .banks_client
            .get_account(print_edition.new_edition_pubkey)
            .await
            .unwrap();

        assert!(md_account.is_some());
        assert!(token_account.is_some());
        assert!(print_edition_account.is_some());

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        let error = burn(
            &mut context,
            original_nft.pubkey,
            &payer,
            original_nft.mint.pubkey(),
            original_nft.token.pubkey(),
            master_edition.pubkey,
            None,
        )
        .await
        .unwrap_err();

        assert_custom_error!(error, MetadataError::MasterEditionHasPrints);
    }

    #[tokio::test]
    async fn require_md_account_to_burn_collection_nft() {
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
            panic!("CollectionDetails is not set!");
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
                CollectionDetails::V1 { size } => {
                    assert_eq!(size, 1);
                }
            }
        } else {
            panic!("CollectionDetails is not set");
        }

        // Burn the NFT w/o passing in collection metadata. This should fail.
        let err = burn(
            &mut context,
            collection_item_nft.pubkey,
            &payer,
            collection_item_nft.mint.pubkey(),
            collection_item_nft.token.pubkey(),
            item_master_edition_account.pubkey,
            None,
        )
        .await
        .unwrap_err();

        assert_custom_error!(err, MetadataError::MissingCollectionMetadata);
    }

    #[tokio::test]
    async fn burning_decrements_collection_size() {
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
            panic!("CollectionDetails is not set!");
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

        // Will look here, this is causing the problem.
        let parent_nft_account = get_account(&mut context, &collection_parent_nft.pubkey).await;
        let parent_metadata =
            ProgramMetadata::deserialize(&mut parent_nft_account.data.as_slice()).unwrap();

        if let Some(details) = parent_metadata.collection_details {
            match details {
                CollectionDetails::V1 { size } => {
                    assert_eq!(size, 1);
                }
            }
        } else {
            panic!("CollectionDetails is not set!");
        }

        // Burn the NFT
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
                CollectionDetails::V1 { size } => {
                    assert_eq!(size, 0);
                }
            }
        } else {
            panic!("CollectionDetails is not set!");
        }
    }

    #[tokio::test]
    async fn burn_unsized_collection_item() {
        let mut context = program_test().start_with_context().await;

        // Create a Collection Parent NFT without the CollectionDetails struct
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

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Verifying collection
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

        let item_master_edition_account = MasterEditionV2::new(&collection_item_nft);
        item_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        // Burn the NFT
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
    }

    #[tokio::test]
    async fn only_owner_can_burn() {
        let mut context = program_test().start_with_context().await;

        let test_metadata = Metadata::new();
        test_metadata
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

        let master_edition = MasterEditionV2::new(&test_metadata);
        master_edition
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        // Metadata, Master Edition and token account exist.
        let md_account = context
            .banks_client
            .get_account(test_metadata.pubkey)
            .await
            .unwrap();
        let token_account = context
            .banks_client
            .get_account(test_metadata.token.pubkey())
            .await
            .unwrap();
        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap();

        assert!(md_account.is_some());
        assert!(token_account.is_some());
        assert!(master_edition_account.is_some());

        let not_owner = Keypair::new();
        airdrop(&mut context, &not_owner.pubkey(), 1_000_000_000)
            .await
            .unwrap();

        let err = burn(
            &mut context,
            test_metadata.pubkey,
            &not_owner,
            test_metadata.mint.pubkey(),
            test_metadata.token.pubkey(),
            master_edition.pubkey,
            None,
        )
        .await
        .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidOwner);
    }

    #[tokio::test]
    async fn update_authority_cannot_burn() {
        let mut context = program_test().start_with_context().await;

        let name = "Test".to_string();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();
        let creators = None;
        let seller_fee_basis_points = 10;
        let is_mutable = true;
        let freeze_authority = None;
        let collection = None;
        let uses = None;

        let test_metadata = Metadata::new();
        test_metadata
            .create_v2(
                &mut context,
                name.clone(),
                symbol.clone(),
                uri.clone(),
                creators.clone(),
                seller_fee_basis_points,
                is_mutable,
                freeze_authority,
                collection.clone(),
                uses.clone(),
            )
            .await
            .unwrap();

        let master_edition = MasterEditionV2::new(&test_metadata);
        master_edition
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        // Metadata, Master Edition and token account exist.
        let md_account = context
            .banks_client
            .get_account(test_metadata.pubkey)
            .await
            .unwrap();
        let token_account = context
            .banks_client
            .get_account(test_metadata.token.pubkey())
            .await
            .unwrap();
        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap();

        assert!(md_account.is_some());
        assert!(token_account.is_some());
        assert!(master_edition_account.is_some());

        // NFT is created with context payer as the update authority so we need to update this first.
        let new_update_authority = Keypair::new();

        test_metadata
            .change_update_authority(&mut context, new_update_authority.pubkey())
            .await
            .unwrap();

        let err = burn(
            &mut context,
            test_metadata.pubkey,
            &new_update_authority,
            test_metadata.mint.pubkey(),
            test_metadata.token.pubkey(),
            master_edition.pubkey,
            None,
        )
        .await
        .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidOwner);
    }
}
