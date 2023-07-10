#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_metadata::{
    error::MetadataError,
    instruction::{
        builders::VerifyBuilder, DelegateArgs, InstructionBuilder, MetadataDelegateRole,
        VerificationArgs,
    },
    pda::{find_metadata_delegate_record_account, find_token_record_account},
    state::{Collection, CollectionDetails, Creator, TokenStandard},
};
use num_traits::FromPrimitive;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, signature::Keypair, signer::Signer, transaction::Transaction,
    transaction::TransactionError,
};
use utils::*;

mod verify_creator {
    use super::*;

    #[tokio::test]
    async fn metadata_wrong_owner() {
        let mut context = program_test().start_with_context().await;

        // Create and mint item.
        let creator = Keypair::new();
        airdrop(&mut context, &creator.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let creators = Some(vec![Creator {
            address: creator.pubkey(),
            share: 100,
            verified: false,
        }]);

        let mut da = DigitalAsset::new();
        da.create_and_mint_with_creators(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            creators.clone(),
        )
        .await
        .unwrap();

        da.assert_creators_matches_on_chain(&mut context, &creators)
            .await;

        // Verify.
        let args = VerificationArgs::CreatorV1;
        let metadata_wrong_owner = Keypair::new().pubkey();
        let err = da
            .verify(
                &mut context,
                creator,
                args,
                Some(metadata_wrong_owner),
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::IncorrectOwner);

        da.assert_creators_matches_on_chain(&mut context, &creators)
            .await;
    }

    #[tokio::test]
    async fn update_authority_cannot_verify_creator() {
        let mut context = program_test().start_with_context().await;

        // Create and mint item.
        let creator = Keypair::new();
        airdrop(&mut context, &creator.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let creators = Some(vec![Creator {
            address: creator.pubkey(),
            share: 100,
            verified: false,
        }]);

        let mut da = DigitalAsset::new();
        da.create_and_mint_with_creators(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            creators.clone(),
        )
        .await
        .unwrap();

        da.assert_creators_matches_on_chain(&mut context, &creators)
            .await;

        // Verify.
        let payer = context.payer.dirty_clone();
        let args = VerificationArgs::CreatorV1;
        let err = da
            .verify(&mut context, payer, args, None, None, None, None, None)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::CreatorNotFound);

        da.assert_creators_matches_on_chain(&mut context, &creators)
            .await;
    }

    #[tokio::test]
    async fn no_creators_found() {
        let mut context = program_test().start_with_context().await;

        // Create and mint item.
        let mut da = DigitalAsset::new();
        da.create_and_mint_with_creators(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            None,
        )
        .await
        .unwrap();

        da.assert_creators_matches_on_chain(&mut context, &None)
            .await;

        // Verify.
        let payer = context.payer.dirty_clone();
        let args = VerificationArgs::CreatorV1;
        let err = da
            .verify(&mut context, payer, args, None, None, None, None, None)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::NoCreatorsPresentOnMetadata);

        da.assert_creators_matches_on_chain(&mut context, &None)
            .await;
    }

    #[tokio::test]
    async fn pass() {
        let mut context = program_test().start_with_context().await;

        // Create and mint item.
        let creator = Keypair::new();
        airdrop(&mut context, &creator.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let creators = Some(vec![Creator {
            address: creator.pubkey(),
            share: 100,
            verified: false,
        }]);

        let mut da = DigitalAsset::new();
        da.create_and_mint_with_creators(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            creators.clone(),
        )
        .await
        .unwrap();

        da.assert_creators_matches_on_chain(&mut context, &creators)
            .await;

        // Verify.
        let args = VerificationArgs::CreatorV1;

        let verified_creators = Some(vec![Creator {
            address: creator.pubkey(),
            share: 100,
            verified: true,
        }]);

        da.verify(&mut context, creator, args, None, None, None, None, None)
            .await
            .unwrap();

        da.assert_creators_matches_on_chain(&mut context, &verified_creators)
            .await;
    }
}

mod verify_collection {
    use super::*;

    #[tokio::test]
    async fn delegate_record_wrong_owner() {
        // See `collections_standard_delegate_cannot_verify()`.
    }

    #[tokio::test]
    async fn metadata_wrong_owner() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
        let (collection_parent_nft, parent_master_edition_account) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let metadata_wrong_owner = Keypair::new().pubkey();
        let err = da
            .verify(
                &mut context,
                payer,
                args,
                Some(metadata_wrong_owner),
                None,
                Some(collection_parent_nft.mint.pubkey()),
                Some(collection_parent_nft.pubkey),
                Some(parent_master_edition_account.pubkey),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::IncorrectOwner);

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn collection_mint_info_wrong_owner() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
        let (collection_parent_nft, parent_master_edition_account) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let collection_mint_info_wrong_owner = Keypair::new().pubkey();
        let err = da
            .verify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(collection_mint_info_wrong_owner),
                Some(collection_parent_nft.pubkey),
                Some(parent_master_edition_account.pubkey),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::IncorrectOwner);

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn collection_metadata_info_wrong_owner() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
        let (collection_parent_nft, parent_master_edition_account) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let collection_metadata_info_wrong_owner = Keypair::new().pubkey();
        let err = da
            .verify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(collection_parent_nft.mint.pubkey()),
                Some(collection_metadata_info_wrong_owner),
                Some(parent_master_edition_account.pubkey),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::IncorrectOwner);

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn collection_master_edition_info_wrong_owner() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
        let (collection_parent_nft, _) = Metadata::create_default_sized_parent(&mut context)
            .await
            .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let collection_master_edition_info_wrong_owner = Keypair::new().pubkey();
        let err = da
            .verify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(collection_parent_nft.mint.pubkey()),
                Some(collection_parent_nft.pubkey),
                Some(collection_master_edition_info_wrong_owner),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::IncorrectOwner);

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn missing_collection_mint_info() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
        let (collection_parent_nft, parent_master_edition_account) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let err = da
            .verify(
                &mut context,
                payer,
                args,
                None,
                None,
                None,
                Some(collection_parent_nft.pubkey),
                Some(parent_master_edition_account.pubkey),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::MissingCollectionMint);

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn missing_collection_metadata_info() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
        let (collection_parent_nft, parent_master_edition_account) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let err = da
            .verify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(collection_parent_nft.mint.pubkey()),
                None,
                Some(parent_master_edition_account.pubkey),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::MissingCollectionMetadata);

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn missing_collection_master_edition_info() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
        let (collection_parent_nft, _) = Metadata::create_default_sized_parent(&mut context)
            .await
            .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let err = da
            .verify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(collection_parent_nft.mint.pubkey()),
                Some(collection_parent_nft.pubkey),
                None,
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::MissingCollectionMasterEdition);

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_already_verified() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
        let (collection_parent_nft, parent_master_edition_account) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        da.verify(
            &mut context,
            payer,
            args,
            None,
            None,
            Some(collection_parent_nft.mint.pubkey()),
            Some(collection_parent_nft.pubkey),
            Some(parent_master_edition_account.pubkey),
        )
        .await
        .unwrap();

        assert_collection_verified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Skip ahead.
        context.warp_to_slot(2).unwrap();

        // Verify again.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        da.verify(
            &mut context,
            payer,
            args,
            None,
            None,
            Some(collection_parent_nft.mint.pubkey()),
            Some(collection_parent_nft.pubkey),
            Some(parent_master_edition_account.pubkey),
        )
        .await
        .unwrap();

        assert_collection_verified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn collection_not_found_on_item() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
        let (collection_parent_nft, parent_master_edition_account) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        // No collection on item's metadata.
        let collection = None;

        // Create and mint item.
        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let err = da
            .verify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(collection_parent_nft.mint.pubkey()),
                Some(collection_parent_nft.pubkey),
                Some(parent_master_edition_account.pubkey),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::CollectionNotFound);

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn item_collection_key_does_not_match_passed_in_collection_mint() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
        let (collection_parent_nft, parent_master_edition_account) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        // Use a different collection key for the item.
        let collection = Some(Collection {
            key: Keypair::new().pubkey(),
            verified: false,
        });

        // Create and mint item.
        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let err = da
            .verify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(collection_parent_nft.mint.pubkey()),
                Some(collection_parent_nft.pubkey),
                Some(parent_master_edition_account.pubkey),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::CollectionNotFound);

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn collection_metadata_mint_does_not_match_passed_in_collection_mint() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
        let (collection_parent_nft, parent_master_edition_account) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        // Create a second Collection Parent NFT with the CollectionDetails struct populated
        let (second_collection_parent_nft, _) = Metadata::create_default_sized_parent(&mut context)
            .await
            .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Check second collection details.
        let second_collection_metadata = second_collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            second_collection_metadata.collection_details,
            DEFAULT_COLLECTION_DETAILS
        );

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let err = da
            .verify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(second_collection_parent_nft.mint.pubkey()),
                Some(collection_parent_nft.pubkey),
                Some(parent_master_edition_account.pubkey),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::CollectionNotFound);

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Check second collection details.
        let second_collection_metadata = second_collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            second_collection_metadata.collection_details,
            DEFAULT_COLLECTION_DETAILS
        );
    }

    #[tokio::test]
    async fn wrong_collection_master_edition() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
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
                DEFAULT_COLLECTION_DETAILS,
            )
            .await
            .unwrap();

        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        // Create a second collection parent NFT.
        let second_collection_parent_nft = Metadata::new();
        second_collection_parent_nft
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
                DEFAULT_COLLECTION_DETAILS,
            )
            .await
            .unwrap();

        let second_parent_master_edition_account =
            MasterEditionV2::new(&second_collection_parent_nft);
        second_parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Check second collection details.
        let second_collection_metadata = second_collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            second_collection_metadata.collection_details,
            DEFAULT_COLLECTION_DETAILS
        );

        // Verify using second collection's master edition account.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let err = da
            .verify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(collection_parent_nft.mint.pubkey()),
                Some(collection_parent_nft.pubkey),
                Some(second_parent_master_edition_account.pubkey),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::CollectionMasterEditionAccountInvalid);

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Check second collection details.
        let second_collection_metadata = second_collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            second_collection_metadata.collection_details,
            DEFAULT_COLLECTION_DETAILS
        );
    }

    #[tokio::test]
    async fn fail_collection_master_edition_has_nonzero_max_supply() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
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
                DEFAULT_COLLECTION_DETAILS,
            )
            .await
            .unwrap();

        // Create a parent master edition with a nonzero max supply.
        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(33))
            .await
            .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let err = da
            .verify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(collection_parent_nft.mint.pubkey()),
                Some(collection_parent_nft.pubkey),
                Some(parent_master_edition_account.pubkey),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::CollectionMustBeAUniqueMasterEdition);

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn incorrect_collection_update_authority() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
        let (collection_parent_nft, parent_master_edition_account) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Create a keypair to use instead of the collection update authority.
        let incorrect_update_authority = Keypair::new();
        airdrop(
            &mut context,
            &incorrect_update_authority.pubkey(),
            LAMPORTS_PER_SOL,
        )
        .await
        .unwrap();

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let err = da
            .verify(
                &mut context,
                incorrect_update_authority,
                args,
                None,
                None,
                Some(collection_parent_nft.mint.pubkey()),
                Some(collection_parent_nft.pubkey),
                Some(parent_master_edition_account.pubkey),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_unsized_collection_item_nft_collection_nft_both_old_handlers_update_authority() {
        pass_item_nft_collection_nft_both_old_handlers_collection_update_authority(None).await;
    }

    #[tokio::test]
    async fn pass_sized_collection_item_nft_collection_nft_both_old_handlers_update_authority() {
        pass_item_nft_collection_nft_both_old_handlers_collection_update_authority(
            DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    async fn pass_item_nft_collection_nft_both_old_handlers_collection_update_authority(
        collection_details: Option<CollectionDetails>,
    ) {
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
                collection_details.clone(), // Collection Parent
            )
            .await
            .unwrap();

        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        // Check collection details.
        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(collection_metadata.collection_details, collection_details);

        // Create item using old handler.
        let collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        });

        let name = "Test".to_string();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();
        let test_metadata = Metadata::new();
        test_metadata
            .create_v3(
                &mut context,
                name,
                symbol,
                uri,
                None,
                10,
                false,
                collection,
                None,
                None,
            )
            .await
            .unwrap();

        // Check metadata unverified collection.
        let metadata = test_metadata.get_data(&mut context).await;
        assert_eq!(
            metadata.collection.to_owned().unwrap().key,
            collection_parent_nft.mint.pubkey()
        );
        assert!(!metadata.collection.unwrap().verified);

        // Check collection details.
        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(collection_metadata.collection_details, collection_details);

        // Build verify instruction since not using DigitalAsset.
        let mut builder = VerifyBuilder::new();
        builder
            .authority(context.payer.pubkey())
            .metadata(test_metadata.pubkey)
            .collection_mint(collection_parent_nft.mint.pubkey())
            .collection_metadata(collection_parent_nft.pubkey)
            .collection_master_edition(parent_master_edition_account.pubkey);

        // Verify.
        let verify_ix = builder
            .build(VerificationArgs::CollectionV1)
            .unwrap()
            .instruction();

        let transaction = Transaction::new_signed_with_payer(
            &[verify_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();

        // Check metadata verified collection.
        let metadata = test_metadata.get_data(&mut context).await;
        assert_eq!(
            metadata.collection.to_owned().unwrap().key,
            collection_parent_nft.mint.pubkey()
        );
        assert!(metadata.collection.unwrap().verified);

        // Check collection details.  If sized collection, size should be updated.
        let verified_collection_details = collection_details.map(|details| match details {
            #[allow(deprecated)]
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            verified_collection_details
        );
    }

    #[tokio::test]
    async fn pass_unsized_collection_item_pnft_collection_nft_old_handler_update_authority() {
        pass_item_pnft_collection_nft_old_handler_collection_update_authority(None).await;
    }

    #[tokio::test]
    async fn pass_sized_collection_item_pnft_collection_nft_old_handler_update_authority() {
        pass_item_pnft_collection_nft_old_handler_collection_update_authority(
            DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    async fn pass_item_pnft_collection_nft_old_handler_collection_update_authority(
        collection_details: Option<CollectionDetails>,
    ) {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
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
                collection_details.clone(), // Collection Parent
            )
            .await
            .unwrap();

        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &collection_details,
        )
        .await;

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        da.verify(
            &mut context,
            payer,
            args,
            None,
            None,
            Some(collection_parent_nft.mint.pubkey()),
            Some(collection_parent_nft.pubkey),
            Some(parent_master_edition_account.pubkey),
        )
        .await
        .unwrap();

        assert_collection_verified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &collection_details,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_with_changed_collection_update_authority() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
        let (collection_parent_nft, parent_master_edition_account) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        // Change the collection to have a different update authority.
        let new_collection_update_authority = Keypair::new();
        collection_parent_nft
            .change_update_authority(&mut context, new_collection_update_authority.pubkey())
            .await
            .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Verify using the new collection update authority.
        let args = VerificationArgs::CollectionV1;
        da.verify(
            &mut context,
            new_collection_update_authority,
            args,
            None,
            None,
            Some(collection_parent_nft.mint.pubkey()),
            Some(collection_parent_nft.pubkey),
            Some(parent_master_edition_account.pubkey),
        )
        .await
        .unwrap();

        assert_collection_verified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn item_update_authority_cannot_verify() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
        let (collection_parent_nft, parent_master_edition_account) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        // Change the collection to have a different update authority.
        let new_collection_update_authority = Keypair::new();
        collection_parent_nft
            .change_update_authority(&mut context, new_collection_update_authority.pubkey())
            .await
            .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Verify using item update authority.
        let payer = context.payer.dirty_clone();
        let args = VerificationArgs::CollectionV1;
        let err = da
            .verify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(collection_parent_nft.mint.pubkey()),
                Some(collection_parent_nft.pubkey),
                Some(parent_master_edition_account.pubkey),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_nft,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    async fn assert_collection_unverified_item_and_parent(
        context: &mut ProgramTestContext,
        item_da: &DigitalAsset,
        collection: &Option<Collection>,
        collection_parent_nft: &Metadata,
        collection_details: &Option<CollectionDetails>,
    ) {
        item_da
            .assert_item_collection_matches_on_chain(context, collection)
            .await;

        let collection_metadata = collection_parent_nft.get_data(context).await;
        assert_eq!(collection_metadata.collection_details, *collection_details);
    }

    async fn assert_collection_verified_item_and_parent(
        context: &mut ProgramTestContext,
        item_da: &DigitalAsset,
        collection: &Option<Collection>,
        collection_parent_nft: &Metadata,
        collection_details: &Option<CollectionDetails>,
    ) {
        let verified_collection = collection.clone().map(|mut c| {
            c.verified = true;
            c
        });

        item_da
            .assert_item_collection_matches_on_chain(context, &verified_collection)
            .await;

        // Check collection details.  If sized collection, size should be updated.
        let verified_collection_details = collection_details.clone().map(|details| match details {
            #[allow(deprecated)]
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        let collection_metadata = collection_parent_nft.get_data(context).await;
        assert_eq!(
            collection_metadata.collection_details,
            verified_collection_details
        );
    }

    #[tokio::test]
    async fn pass_unsized_collection_item_nft_collection_nft_new_handler_update_authority() {
        pass_collection_new_handler_collection_update_authority(
            None,
            TokenStandard::NonFungible,
            TokenStandard::NonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_unsized_collection_item_nft_collection_pnft_new_handler_update_authority() {
        pass_collection_new_handler_collection_update_authority(
            None,
            TokenStandard::NonFungible,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_unsized_collection_item_pnft_collection_nft_new_handler_update_authority() {
        pass_collection_new_handler_collection_update_authority(
            None,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::NonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_unsized_collection_item_pnft_collection_pnft_new_handler_update_authority() {
        pass_collection_new_handler_collection_update_authority(
            None,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_sized_collection_item_nft_collection_nft_new_handler_update_authority() {
        pass_collection_new_handler_collection_update_authority(
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::NonFungible,
            TokenStandard::NonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_sized_collection_item_nft_collection_pnft_new_handler_update_authority() {
        pass_collection_new_handler_collection_update_authority(
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::NonFungible,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_sized_collection_item_pnft_collection_nft_new_handler_update_authority() {
        pass_collection_new_handler_collection_update_authority(
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::NonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_sized_collection_item_pnft_collection_pnft_new_handler_update_authority() {
        pass_collection_new_handler_collection_update_authority(
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;
    }

    async fn pass_collection_new_handler_collection_update_authority(
        collection_details: Option<CollectionDetails>,
        item_token_standard: TokenStandard,
        collection_token_standard: TokenStandard,
    ) {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT or pNFT with the CollectionDetails struct populated.
        let mut collection_parent_da = DigitalAsset::new();
        collection_parent_da
            .create_and_mint_collection_parent(
                &mut context,
                collection_token_standard,
                None,
                None,
                1,
                collection_details.clone(),
            )
            .await
            .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_da.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            item_token_standard,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &collection_details)
            .await;

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        da.verify(
            &mut context,
            payer,
            args,
            None,
            None,
            Some(collection_parent_da.mint.pubkey()),
            Some(collection_parent_da.metadata),
            Some(collection_parent_da.edition.unwrap()),
        )
        .await
        .unwrap();

        let verified_collection = Some(Collection {
            key: collection_parent_da.mint.pubkey(),
            verified: true,
        });

        da.assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;

        // Check collection details.  If sized collection, size should be updated.
        let verified_collection_details = collection_details.map(|details| match details {
            #[allow(deprecated)]
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &verified_collection_details)
            .await;
    }

    #[tokio::test]
    async fn pass_unsized_collection_item_nft_collection_nft_new_handler_delegated_authority() {
        pass_collection_new_handler_delegated_authority(
            None,
            TokenStandard::NonFungible,
            TokenStandard::NonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_unsized_collection_item_nft_collection_pnft_new_handler_delegated_authority() {
        pass_collection_new_handler_delegated_authority(
            None,
            TokenStandard::NonFungible,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_unsized_collection_item_pnft_collection_nft_new_handler_delegated_authority() {
        pass_collection_new_handler_delegated_authority(
            None,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::NonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_unsized_collection_item_pnft_collection_pnft_new_handler_delegated_authority() {
        pass_collection_new_handler_delegated_authority(
            None,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_sized_collection_item_nft_collection_nft_new_handler_delegated_authority() {
        pass_collection_new_handler_delegated_authority(
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::NonFungible,
            TokenStandard::NonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_sized_collection_item_nft_collection_pnft_new_handler_delegated_authority() {
        pass_collection_new_handler_delegated_authority(
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::NonFungible,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_sized_collection_item_pnft_collection_nft_new_handler_delegated_authority() {
        pass_collection_new_handler_delegated_authority(
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::NonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_sized_collection_item_pnft_collection_pnft_new_handler_delegated_authority() {
        pass_collection_new_handler_delegated_authority(
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;
    }

    async fn pass_collection_new_handler_delegated_authority(
        collection_details: Option<CollectionDetails>,
        item_token_standard: TokenStandard,
        collection_token_standard: TokenStandard,
    ) {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT or pNFT with the CollectionDetails struct populated.
        let mut collection_parent_da = DigitalAsset::new();
        collection_parent_da
            .create_and_mint_collection_parent(
                &mut context,
                collection_token_standard,
                None,
                None,
                1,
                collection_details.clone(),
            )
            .await
            .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_da.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            item_token_standard,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &collection_details)
            .await;

        // Create a Collection delegate.
        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let payer = context.payer.dirty_clone();
        let payer_pubkey = payer.pubkey();
        collection_parent_da
            .delegate(
                &mut context,
                payer,
                delegate.pubkey(),
                DelegateArgs::CollectionV1 {
                    authorization_data: None,
                },
            )
            .await
            .unwrap();

        // Find delegate record PDA.
        let (delegate_record, _) = find_metadata_delegate_record_account(
            &collection_parent_da.mint.pubkey(),
            MetadataDelegateRole::Collection,
            &payer_pubkey,
            &delegate.pubkey(),
        );

        // Verify.
        let args = VerificationArgs::CollectionV1;
        da.verify(
            &mut context,
            delegate,
            args,
            None,
            Some(delegate_record),
            Some(collection_parent_da.mint.pubkey()),
            Some(collection_parent_da.metadata),
            Some(collection_parent_da.edition.unwrap()),
        )
        .await
        .unwrap();

        let verified_collection = Some(Collection {
            key: collection_parent_da.mint.pubkey(),
            verified: true,
        });

        da.assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;

        // Check collection details.  If sized collection, size should be updated.
        let verified_collection_details = collection_details.map(|details| match details {
            #[allow(deprecated)]
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &verified_collection_details)
            .await;
    }

    #[tokio::test]
    async fn collections_collection_item_delegate_cannot_verify() {
        let delegate_args = DelegateArgs::CollectionItemV1 {
            authorization_data: None,
        };

        let delegate_role = MetadataDelegateRole::CollectionItem;

        other_metadata_delegates_cannot_verify(
            AssetToDelegate::CollectionParent,
            delegate_args,
            delegate_role,
        )
        .await;
    }

    #[tokio::test]
    async fn collections_programmable_config_delegate_cannot_verify() {
        let delegate_args = DelegateArgs::ProgrammableConfigV1 {
            authorization_data: None,
        };

        let delegate_role = MetadataDelegateRole::ProgrammableConfig;

        other_metadata_delegates_cannot_verify(
            AssetToDelegate::CollectionParent,
            delegate_args,
            delegate_role,
        )
        .await;
    }

    #[tokio::test]
    async fn items_collection_delegate_cannot_verify() {
        let delegate_args = DelegateArgs::CollectionV1 {
            authorization_data: None,
        };

        let delegate_role = MetadataDelegateRole::Collection;

        other_metadata_delegates_cannot_verify(AssetToDelegate::Item, delegate_args, delegate_role)
            .await;
    }

    #[tokio::test]
    async fn items_collection_item_delegate_cannot_verify() {
        let delegate_args = DelegateArgs::CollectionItemV1 {
            authorization_data: None,
        };

        let delegate_role = MetadataDelegateRole::CollectionItem;

        other_metadata_delegates_cannot_verify(AssetToDelegate::Item, delegate_args, delegate_role)
            .await;
    }

    #[tokio::test]
    async fn items_programmable_config_delegate_cannot_verify() {
        let delegate_args = DelegateArgs::ProgrammableConfigV1 {
            authorization_data: None,
        };

        let delegate_role = MetadataDelegateRole::ProgrammableConfig;

        other_metadata_delegates_cannot_verify(AssetToDelegate::Item, delegate_args, delegate_role)
            .await;
    }

    enum AssetToDelegate {
        CollectionParent,
        Item,
    }

    async fn other_metadata_delegates_cannot_verify(
        asset_to_delegate: AssetToDelegate,
        delegate_args: DelegateArgs,
        delegate_role: MetadataDelegateRole,
    ) {
        let mut context = program_test().start_with_context().await;

        // Use pNFT for collection parent for this test.
        let mut collection_parent_da = DigitalAsset::new();
        collection_parent_da
            .create_and_mint_collection_parent(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
                DEFAULT_COLLECTION_DETAILS,
            )
            .await
            .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_da.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &DEFAULT_COLLECTION_DETAILS)
            .await;

        // Create a metadata delegate.
        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let asset = match asset_to_delegate {
            AssetToDelegate::CollectionParent => &mut collection_parent_da,
            AssetToDelegate::Item => &mut da,
        };

        let payer = context.payer.dirty_clone();
        let payer_pubkey = payer.pubkey();
        asset
            .delegate(&mut context, payer, delegate.pubkey(), delegate_args)
            .await
            .unwrap();

        // Find delegate record PDA.
        let (delegate_record, _) = find_metadata_delegate_record_account(
            &asset.mint.pubkey(),
            delegate_role,
            &payer_pubkey,
            &delegate.pubkey(),
        );

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let err = da
            .verify(
                &mut context,
                delegate,
                args,
                None,
                Some(delegate_record),
                Some(collection_parent_da.mint.pubkey()),
                Some(collection_parent_da.metadata),
                Some(collection_parent_da.edition.unwrap()),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Check collection details.  It should not be updated.
        collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &DEFAULT_COLLECTION_DETAILS)
            .await;
    }

    #[tokio::test]
    async fn delegate_for_different_collection_cannot_verify() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent pNFT with the CollectionDetails struct populated.
        let mut first_collection_parent_da = DigitalAsset::new();
        first_collection_parent_da
            .create_and_mint_collection_parent(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
                DEFAULT_COLLECTION_DETAILS,
            )
            .await
            .unwrap();

        // Create a second collection parent pNFT with the CollectionDetails struct populated.
        let mut second_collection_parent_da = DigitalAsset::new();
        second_collection_parent_da
            .create_and_mint_collection_parent(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
                DEFAULT_COLLECTION_DETAILS,
            )
            .await
            .unwrap();

        // Make the item a member of the first collection.
        let collection = Some(Collection {
            key: first_collection_parent_da.mint.pubkey(),
            verified: false,
        });

        // Create and mint item.
        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Check collection details for each collection.
        first_collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &DEFAULT_COLLECTION_DETAILS)
            .await;

        second_collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &DEFAULT_COLLECTION_DETAILS)
            .await;

        // Create a Collection delegate for the second collection.
        let second_collection_delegate = Keypair::new();
        airdrop(
            &mut context,
            &second_collection_delegate.pubkey(),
            LAMPORTS_PER_SOL,
        )
        .await
        .unwrap();

        let payer = context.payer.dirty_clone();
        let payer_pubkey = payer.pubkey();
        second_collection_parent_da
            .delegate(
                &mut context,
                payer,
                second_collection_delegate.pubkey(),
                DelegateArgs::CollectionV1 {
                    authorization_data: None,
                },
            )
            .await
            .unwrap();

        // Find delegate record PDA.
        let (second_collection_delegate_record, _) = find_metadata_delegate_record_account(
            &second_collection_parent_da.mint.pubkey(),
            MetadataDelegateRole::Collection,
            &payer_pubkey,
            &second_collection_delegate.pubkey(),
        );

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let err = da
            .verify(
                &mut context,
                second_collection_delegate,
                args,
                None,
                Some(second_collection_delegate_record),
                Some(first_collection_parent_da.mint.pubkey()),
                Some(first_collection_parent_da.metadata),
                Some(first_collection_parent_da.edition.unwrap()),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Check collection details.  They should not be updated.
        first_collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &DEFAULT_COLLECTION_DETAILS)
            .await;

        second_collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &DEFAULT_COLLECTION_DETAILS)
            .await;
    }

    #[tokio::test]
    async fn collections_standard_delegate_cannot_verify() {
        let mut context = program_test().start_with_context().await;

        // Use NFT for collection parent for this test.
        let mut collection_parent_da = DigitalAsset::new();
        collection_parent_da
            .create_and_mint_collection_parent(
                &mut context,
                TokenStandard::NonFungible,
                None,
                None,
                1,
                DEFAULT_COLLECTION_DETAILS,
            )
            .await
            .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_da.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &DEFAULT_COLLECTION_DETAILS)
            .await;

        // Create a Standard delegate.
        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let payer = context.payer.dirty_clone();
        let delegate_args = DelegateArgs::StandardV1 { amount: 1 };
        collection_parent_da
            .delegate(&mut context, payer, delegate.pubkey(), delegate_args)
            .await
            .unwrap();

        // This account was not actually created by the delegate instruction but we will send
        // it anyways and expect to see an `IncorrectOwner` failure.
        let (token_record, _) = find_token_record_account(
            &collection_parent_da.mint.pubkey(),
            &collection_parent_da.token.unwrap(),
        );

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let err = da
            .verify(
                &mut context,
                delegate,
                args,
                None,
                Some(token_record),
                Some(collection_parent_da.mint.pubkey()),
                Some(collection_parent_da.metadata),
                Some(collection_parent_da.edition.unwrap()),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::IncorrectOwner);

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Check collection details.  It should not be updated.
        collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &DEFAULT_COLLECTION_DETAILS)
            .await;
    }

    #[tokio::test]
    async fn collections_utility_delegate_cannot_verify() {
        utility_delegate_cannot_verify(AssetToDelegate::CollectionParent).await;
    }

    #[tokio::test]
    async fn items_utility_delegate_cannot_verify() {
        utility_delegate_cannot_verify(AssetToDelegate::Item).await;
    }

    async fn utility_delegate_cannot_verify(asset_to_delegate: AssetToDelegate) {
        let mut context = program_test().start_with_context().await;

        // Use pNFT for collection parent for this test.
        let mut collection_parent_da = DigitalAsset::new();
        collection_parent_da
            .create_and_mint_collection_parent(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
                DEFAULT_COLLECTION_DETAILS,
            )
            .await
            .unwrap();

        // Create and mint item.
        let collection = Some(Collection {
            key: collection_parent_da.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &DEFAULT_COLLECTION_DETAILS)
            .await;

        // Create a Utility delegate.
        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let asset = match asset_to_delegate {
            AssetToDelegate::CollectionParent => &mut collection_parent_da,
            AssetToDelegate::Item => &mut da,
        };

        let payer = context.payer.dirty_clone();
        let delegate_args = DelegateArgs::UtilityV1 {
            amount: 1,
            authorization_data: None,
        };
        asset
            .delegate(&mut context, payer, delegate.pubkey(), delegate_args)
            .await
            .unwrap();

        // Find the token_record account for the Utility Delegate.
        let (token_record, _) =
            find_token_record_account(&asset.mint.pubkey(), &asset.token.unwrap());

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let err = da
            .verify(
                &mut context,
                delegate,
                args,
                None,
                Some(token_record),
                Some(collection_parent_da.mint.pubkey()),
                Some(collection_parent_da.metadata),
                Some(collection_parent_da.edition.unwrap()),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Check collection details.  It should not be updated.
        collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &DEFAULT_COLLECTION_DETAILS)
            .await;
    }
}
