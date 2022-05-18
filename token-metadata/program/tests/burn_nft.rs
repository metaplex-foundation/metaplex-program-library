#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::state::Metadata as ProgramMetadata;
use num_traits::FromPrimitive;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, signer::Signer, transaction::TransactionError,
    transport::TransportError,
};
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

        burn(
            &mut context,
            test_metadata.pubkey,
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
    async fn successfully_burn_print_edition_nft() {
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

        burn(
            &mut context,
            print_edition.new_metadata_pubkey,
            print_edition.mint.pubkey(),
            print_edition.token.pubkey(),
            print_edition.new_edition_pubkey,
            None,
        )
        .await
        .unwrap();

        // Metadata, Master Edition and token account are burned.
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

        assert!(md_account.is_none());
        assert!(token_account.is_none());
        assert!(print_edition_account.is_none());
    }

    #[tokio::test]
    async fn require_md_account_to_burn_collection_nft() {
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
                true, // Collection Parent
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
                false, // Collection Item
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

        if let CollectionDetails::CollectionDetailsV1 { status: _, size } =
            parent_metadata.collection_details
        {
            assert_eq!(size, 0);
        } else {
            panic!("CollectionDetails is not a CollectionDetails");
        }

        // Verifying increments the size.
        collection_item_nft
            .verify_collection_v2(
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

        if let CollectionDetails::CollectionDetailsV1 { status: _, size } =
            parent_metadata.collection_details
        {
            assert_eq!(size, 1);
        } else {
            panic!("CollectionDetails is not a CollectionDetails");
        }

        // Burn the NFT w/o passing in collection metadata. This should fail.
        let err = burn(
            &mut context,
            collection_item_nft.pubkey,
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
                true, // Collection Parent
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
                false, // Collection Item
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

        if let CollectionDetails::CollectionDetailsV1 { status: _, size } =
            parent_metadata.collection_details
        {
            assert_eq!(size, 0);
        } else {
            panic!("CollectionDetails is not a CollectionDetails");
        }

        // Verifying increments the size.
        collection_item_nft
            .verify_collection_v2(
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

        if let CollectionDetails::CollectionDetailsV1 { status: _, size } =
            parent_metadata.collection_details
        {
            assert_eq!(size, 1);
        } else {
            panic!("CollectionDetails is not a CollectionDetails");
        }

        // Burn the NFT
        burn(
            &mut context,
            collection_item_nft.pubkey,
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

        if let CollectionDetails::CollectionDetailsV1 { status: _, size } =
            parent_metadata.collection_details
        {
            assert_eq!(size, 0);
        } else {
            panic!("CollectionDetails is not a CollectionDetails");
        }
    }
}
