#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_metadata::{
    error::MetadataError,
    instruction::{BurnArgs, DelegateArgs, MetadataDelegateRole, UpdateArgs, VerificationArgs},
    pda::{find_metadata_delegate_record_account, find_token_record_account},
    state::{Collection, CollectionDetails, Creator, TokenStandard},
};
use num_traits::FromPrimitive;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, signature::Keypair, signer::Signer,
    transaction::TransactionError,
};
use utils::*;

mod unverify_creator {
    use super::*;

    #[tokio::test]
    async fn metadata_wrong_owner() {
        let mut context = program_test().start_with_context().await;
        let mut da = DigitalAsset::new();

        let creator = Keypair::new();
        airdrop(&mut context, &creator.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let unverified_creators = Some(vec![Creator {
            address: creator.pubkey(),
            share: 100,
            verified: false,
        }]);

        // Create, mint, verify creator, and check creators matches on-chain.
        create_mint_verify_creator_check(
            &mut context,
            &mut da,
            creator.dirty_clone(),
            &unverified_creators,
        )
        .await;

        let verified_creators = Some(vec![Creator {
            address: creator.pubkey(),
            share: 100,
            verified: true,
        }]);

        // Unverify creator.
        let args = VerificationArgs::CreatorV1;
        let metadata_wrong_owner = Keypair::new().pubkey();
        let err = da
            .unverify(
                &mut context,
                creator,
                args,
                Some(metadata_wrong_owner),
                None,
                None,
                None,
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::IncorrectOwner);

        da.assert_creators_matches_on_chain(&mut context, &verified_creators)
            .await;
    }

    #[tokio::test]
    async fn update_authority_cannot_unverify_creator() {
        let mut context = program_test().start_with_context().await;
        let mut da = DigitalAsset::new();

        let creator = Keypair::new();
        airdrop(&mut context, &creator.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let unverified_creators = Some(vec![Creator {
            address: creator.pubkey(),
            share: 100,
            verified: false,
        }]);

        // Create, mint, verify creator, and check creators matches on-chain.
        create_mint_verify_creator_check(
            &mut context,
            &mut da,
            creator.dirty_clone(),
            &unverified_creators,
        )
        .await;

        let verified_creators = Some(vec![Creator {
            address: creator.pubkey(),
            share: 100,
            verified: true,
        }]);

        // Unverify creator.
        let payer = context.payer.dirty_clone();
        let args = VerificationArgs::CreatorV1;
        let err = da
            .unverify(&mut context, payer, args, None, None, None, None)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::CreatorNotFound);

        da.assert_creators_matches_on_chain(&mut context, &verified_creators)
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

        // Unverify creator.
        let payer = context.payer.dirty_clone();
        let args = VerificationArgs::CreatorV1;
        let err = da
            .unverify(&mut context, payer, args, None, None, None, None)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::NoCreatorsPresentOnMetadata);

        da.assert_creators_matches_on_chain(&mut context, &None)
            .await;
    }

    #[tokio::test]
    async fn pass() {
        let mut context = program_test().start_with_context().await;
        let mut da = DigitalAsset::new();

        let creator = Keypair::new();
        airdrop(&mut context, &creator.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let unverified_creators = Some(vec![Creator {
            address: creator.pubkey(),
            share: 100,
            verified: false,
        }]);

        // Create, mint, verify creator, and check creators matches on-chain.
        create_mint_verify_creator_check(
            &mut context,
            &mut da,
            creator.dirty_clone(),
            &unverified_creators,
        )
        .await;

        // Unverify creator.
        let args = VerificationArgs::CreatorV1;

        da.unverify(&mut context, creator, args, None, None, None, None)
            .await
            .unwrap();

        da.assert_creators_matches_on_chain(&mut context, &unverified_creators)
            .await;
    }

    async fn create_mint_verify_creator_check(
        context: &mut ProgramTestContext,
        da: &mut DigitalAsset,
        creator: Keypair,
        unverified_creators: &Option<Vec<Creator>>,
    ) {
        // Create and mint item.
        da.create_and_mint_with_creators(
            context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            unverified_creators.clone(),
        )
        .await
        .unwrap();

        da.assert_creators_matches_on_chain(context, unverified_creators)
            .await;

        // Verify.
        let args = VerificationArgs::CreatorV1;

        let verified_creators = Some(
            unverified_creators
                .clone()
                .unwrap()
                .into_iter()
                .map(|mut c| {
                    if c.address == creator.pubkey() {
                        c.verified = true
                    }
                    c
                })
                .collect::<Vec<Creator>>(),
        );

        da.verify(
            context,
            creator.dirty_clone(),
            args,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        da.assert_creators_matches_on_chain(context, &verified_creators)
            .await;
    }
}

mod unverify_collection {
    use super::*;

    #[tokio::test]
    async fn delegate_record_wrong_owner() {
        // See `collections_standard_delegate_cannot_unverify()`.
    }

    #[tokio::test]
    async fn metadata_wrong_owner() {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::NonFungible,
        )
        .await;

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let metadata_wrong_owner = Keypair::new().pubkey();
        let err = test_items
            .da
            .unverify(
                &mut context,
                payer,
                args,
                Some(metadata_wrong_owner),
                None,
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::IncorrectOwner);

        assert_collection_verified_item_and_parent(
            &mut context,
            &test_items.da,
            &test_items.collection,
            &test_items.collection_parent_da,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn collection_mint_info_wrong_owner() {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::NonFungible,
        )
        .await;

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let collection_mint_info_wrong_owner = Keypair::new().pubkey();
        let err = test_items
            .da
            .unverify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(collection_mint_info_wrong_owner),
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::IncorrectOwner);

        assert_collection_verified_item_and_parent(
            &mut context,
            &test_items.da,
            &test_items.collection,
            &test_items.collection_parent_da,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn missing_collection_mint_info() {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::NonFungible,
        )
        .await;

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let err = test_items
            .da
            .unverify(
                &mut context,
                payer,
                args,
                None,
                None,
                None,
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::MissingCollectionMint);

        assert_collection_verified_item_and_parent(
            &mut context,
            &test_items.da,
            &test_items.collection,
            &test_items.collection_parent_da,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn missing_collection_metadata_info() {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::NonFungible,
        )
        .await;

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let err = test_items
            .da
            .unverify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(test_items.collection_parent_da.mint.pubkey()),
                None,
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::MissingCollectionMetadata);

        assert_collection_verified_item_and_parent(
            &mut context,
            &test_items.da,
            &test_items.collection,
            &test_items.collection_parent_da,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_no_collection_on_item() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
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
            &collection_parent_da,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        da.unverify(
            &mut context,
            payer,
            args,
            None,
            None,
            Some(collection_parent_da.mint.pubkey()),
            Some(collection_parent_da.metadata),
        )
        .await
        .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &da,
            &collection,
            &collection_parent_da,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_already_unverified() {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::NonFungible,
        )
        .await;

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        test_items
            .da
            .unverify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &test_items.da,
            &test_items.collection,
            &test_items.collection_parent_da,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Skip ahead.
        context.warp_to_slot(2).unwrap();

        // Unverify again.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        test_items
            .da
            .unverify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &test_items.da,
            &test_items.collection,
            &test_items.collection_parent_da,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn collection_on_item_metadata_does_not_match_passed_in_collection_mint() {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::NonFungible,
        )
        .await;

        // Create a second collection parent NFT with the CollectionDetails struct populated.
        let mut second_collection_parent_da = DigitalAsset::new();
        second_collection_parent_da
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

        second_collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &DEFAULT_COLLECTION_DETAILS)
            .await;

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let err = test_items
            .da
            .unverify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(second_collection_parent_da.mint.pubkey()),
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::NotAMemberOfCollection);

        assert_collection_verified_item_and_parent(
            &mut context,
            &test_items.da,
            &test_items.collection,
            &test_items.collection_parent_da,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Second collection's details should not be changed.
        second_collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &DEFAULT_COLLECTION_DETAILS)
            .await;
    }

    #[tokio::test]
    async fn collection_metadata_info_wrong_derivation() {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::NonFungible,
        )
        .await;

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let wrong_collection_metadata = Keypair::new().pubkey();
        let err = test_items
            .da
            .unverify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(wrong_collection_metadata),
            )
            .await
            .unwrap_err();

        // In this case it will be MintMismatch because it fails a derivation check before
        // it gets to an owner check.
        assert_custom_error!(err, MetadataError::MintMismatch);

        assert_collection_verified_item_and_parent(
            &mut context,
            &test_items.da,
            &test_items.collection,
            &test_items.collection_parent_da,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn other_collections_metadata_fails_derivation_check() {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::NonFungible,
        )
        .await;

        // Create a second collection parent NFT with the CollectionDetails struct populated.
        let mut second_collection_parent_da = DigitalAsset::new();
        second_collection_parent_da
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

        second_collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &DEFAULT_COLLECTION_DETAILS)
            .await;

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let err = test_items
            .da
            .unverify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(second_collection_parent_da.metadata),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::MintMismatch);

        assert_collection_verified_item_and_parent(
            &mut context,
            &test_items.da,
            &test_items.collection,
            &test_items.collection_parent_da,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        // Second collection's details should not be changed.
        second_collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &DEFAULT_COLLECTION_DETAILS)
            .await;
    }

    #[tokio::test]
    async fn incorrect_collection_update_authority() {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::NonFungible,
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

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let err = test_items
            .da
            .unverify(
                &mut context,
                incorrect_update_authority,
                args,
                None,
                None,
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);

        assert_collection_verified_item_and_parent(
            &mut context,
            &test_items.da,
            &test_items.collection,
            &test_items.collection_parent_da,
            &DEFAULT_COLLECTION_DETAILS,
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Check collection details.
        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            DEFAULT_COLLECTION_DETAILS
        );

        // Verify.
        let payer = context.payer.dirty_clone();
        let args = VerificationArgs::CollectionV1;
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

        let verified_collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: true,
        });

        da.assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;

        // Check collection details.  If sized collection, size should be updated.
        let verified_collection_details = DEFAULT_COLLECTION_DETAILS.map(|details| match details {
            #[allow(deprecated)]
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            verified_collection_details
        );

        // Change the collection to have a different update authority.
        let new_collection_update_authority = Keypair::new();
        collection_parent_nft
            .change_update_authority(&mut context, new_collection_update_authority.pubkey())
            .await
            .unwrap();

        // Unverify using the new collection update authority.
        let args = VerificationArgs::CollectionV1;
        da.unverify(
            &mut context,
            new_collection_update_authority,
            args,
            None,
            None,
            Some(collection_parent_nft.mint.pubkey()),
            Some(collection_parent_nft.pubkey),
        )
        .await
        .unwrap();

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            DEFAULT_COLLECTION_DETAILS
        );
    }

    #[tokio::test]
    async fn item_update_authority_cannot_unverify() {
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Check collection details.
        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            DEFAULT_COLLECTION_DETAILS
        );

        // Verify.
        let payer = context.payer.dirty_clone();
        let args = VerificationArgs::CollectionV1;
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

        let verified_collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: true,
        });

        da.assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;

        // Check collection details.  If sized collection, size should be updated.
        let verified_collection_details = DEFAULT_COLLECTION_DETAILS.map(|details| match details {
            #[allow(deprecated)]
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            verified_collection_details
        );

        // Change the collection to have a different update authority.
        let new_collection_update_authority = Keypair::new();
        collection_parent_nft
            .change_update_authority(&mut context, new_collection_update_authority.pubkey())
            .await
            .unwrap();

        // Unverify using item update authority.
        let payer = context.payer.dirty_clone();
        let args = VerificationArgs::CollectionV1;
        let err = da
            .unverify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(collection_parent_nft.mint.pubkey()),
                Some(collection_parent_nft.pubkey),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);

        da.assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;

        // Check collection details.  Should have stayed the same.
        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
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

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            collection_details.clone(),
            item_token_standard,
            collection_token_standard,
        )
        .await;

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        test_items
            .da
            .unverify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &test_items.da,
            &test_items.collection,
            &test_items.collection_parent_da,
            &collection_details,
        )
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

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            collection_details.clone(),
            item_token_standard,
            collection_token_standard,
        )
        .await;

        // Create a Collection delegate.
        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let payer = context.payer.dirty_clone();
        let payer_pubkey = payer.pubkey();
        test_items
            .collection_parent_da
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
            &test_items.collection_parent_da.mint.pubkey(),
            MetadataDelegateRole::Collection,
            &payer_pubkey,
            &delegate.pubkey(),
        );

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        test_items
            .da
            .unverify(
                &mut context,
                delegate,
                args,
                None,
                Some(delegate_record),
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap();

        assert_collection_unverified_item_and_parent(
            &mut context,
            &test_items.da,
            &test_items.collection,
            &test_items.collection_parent_da,
            &collection_details,
        )
        .await;
    }

    #[tokio::test]
    async fn collections_collection_item_delegate_cannot_unverify() {
        let delegate_args = DelegateArgs::CollectionItemV1 {
            authorization_data: None,
        };

        let delegate_role = MetadataDelegateRole::CollectionItem;

        other_metadata_delegates_cannot_unverify(
            AssetToDelegate::CollectionParent,
            delegate_args,
            delegate_role,
        )
        .await;
    }

    #[tokio::test]
    async fn collections_programmable_config_delegate_cannot_unverify() {
        let delegate_args = DelegateArgs::ProgrammableConfigV1 {
            authorization_data: None,
        };

        let delegate_role = MetadataDelegateRole::ProgrammableConfig;

        other_metadata_delegates_cannot_unverify(
            AssetToDelegate::CollectionParent,
            delegate_args,
            delegate_role,
        )
        .await;
    }

    #[tokio::test]
    async fn items_collection_delegate_cannot_unverify() {
        let delegate_args = DelegateArgs::CollectionV1 {
            authorization_data: None,
        };

        let delegate_role = MetadataDelegateRole::Collection;

        other_metadata_delegates_cannot_unverify(
            AssetToDelegate::Item,
            delegate_args,
            delegate_role,
        )
        .await;
    }

    #[tokio::test]
    async fn items_collection_item_delegate_cannot_unverify() {
        let delegate_args = DelegateArgs::CollectionItemV1 {
            authorization_data: None,
        };

        let delegate_role = MetadataDelegateRole::CollectionItem;

        other_metadata_delegates_cannot_unverify(
            AssetToDelegate::Item,
            delegate_args,
            delegate_role,
        )
        .await;
    }

    #[tokio::test]
    async fn items_programmable_config_delegate_cannot_unverify() {
        let delegate_args = DelegateArgs::ProgrammableConfigV1 {
            authorization_data: None,
        };

        let delegate_role = MetadataDelegateRole::ProgrammableConfig;

        other_metadata_delegates_cannot_unverify(
            AssetToDelegate::Item,
            delegate_args,
            delegate_role,
        )
        .await;
    }

    enum AssetToDelegate {
        CollectionParent,
        Item,
    }

    async fn other_metadata_delegates_cannot_unverify(
        asset_to_delegate: AssetToDelegate,
        delegate_args: DelegateArgs,
        delegate_role: MetadataDelegateRole,
    ) {
        let mut context = program_test().start_with_context().await;

        // Use pNFT for collection parent for this test.
        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;

        // Create a metadata delegate.
        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let asset = match asset_to_delegate {
            AssetToDelegate::CollectionParent => &mut test_items.collection_parent_da,
            AssetToDelegate::Item => &mut test_items.da,
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

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let err = test_items
            .da
            .unverify(
                &mut context,
                delegate,
                args,
                None,
                Some(delegate_record),
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);

        assert_collection_verified_item_and_parent(
            &mut context,
            &test_items.da,
            &test_items.collection,
            &test_items.collection_parent_da,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn delegate_for_different_collection_cannot_unverify() {
        let mut context = program_test().start_with_context().await;

        // This creates a collection and item and makes the item a member of the first collection.
        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;

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

        second_collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &DEFAULT_COLLECTION_DETAILS)
            .await;

        // Find delegate record PDA.
        let (second_collection_delegate_record, _) = find_metadata_delegate_record_account(
            &second_collection_parent_da.mint.pubkey(),
            MetadataDelegateRole::Collection,
            &payer_pubkey,
            &second_collection_delegate.pubkey(),
        );

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let err = test_items
            .da
            .unverify(
                &mut context,
                second_collection_delegate,
                args,
                None,
                Some(second_collection_delegate_record),
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);

        assert_collection_verified_item_and_parent(
            &mut context,
            &test_items.da,
            &test_items.collection,
            &test_items.collection_parent_da,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;

        second_collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &DEFAULT_COLLECTION_DETAILS)
            .await;
    }

    #[tokio::test]
    async fn collections_standard_delegate_cannot_unverify() {
        let mut context = program_test().start_with_context().await;

        // Use NFT for collection parent for this test.
        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::NonFungible,
        )
        .await;

        // Create a Standard delegate.
        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let payer = context.payer.dirty_clone();
        let delegate_args = DelegateArgs::StandardV1 { amount: 1 };
        test_items
            .collection_parent_da
            .delegate(&mut context, payer, delegate.pubkey(), delegate_args)
            .await
            .unwrap();

        // This account was not actually created by the delegate instruction but we will send
        // it anyways and expect to see an `IncorrectOwner` failure.
        let (token_record, _) = find_token_record_account(
            &test_items.collection_parent_da.mint.pubkey(),
            &test_items.collection_parent_da.token.unwrap(),
        );

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let err = test_items
            .da
            .unverify(
                &mut context,
                delegate,
                args,
                None,
                Some(token_record),
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::IncorrectOwner);

        assert_collection_verified_item_and_parent(
            &mut context,
            &test_items.da,
            &test_items.collection,
            &test_items.collection_parent_da,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn collections_utility_delegate_cannot_unverify() {
        utility_delegate_cannot_unverify(AssetToDelegate::CollectionParent).await;
    }

    #[tokio::test]
    async fn items_utility_delegate_cannot_unverify() {
        utility_delegate_cannot_unverify(AssetToDelegate::Item).await;
    }

    async fn utility_delegate_cannot_unverify(asset_to_delegate: AssetToDelegate) {
        let mut context = program_test().start_with_context().await;

        // Use pNFT for collection parent for this test.
        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;

        // Create a Utility delegate.
        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let asset = match asset_to_delegate {
            AssetToDelegate::CollectionParent => &mut test_items.collection_parent_da,
            AssetToDelegate::Item => &mut test_items.da,
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

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let err = test_items
            .da
            .unverify(
                &mut context,
                delegate,
                args,
                None,
                Some(token_record),
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);

        assert_collection_verified_item_and_parent(
            &mut context,
            &test_items.da,
            &test_items.collection,
            &test_items.collection_parent_da,
            &DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn burned_nft_collections_update_authority_cannot_unverify() {
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Check collection details.
        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            DEFAULT_COLLECTION_DETAILS
        );

        // Verify.
        let payer = context.payer.dirty_clone();
        let args = VerificationArgs::CollectionV1;
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

        let verified_collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: true,
        });

        da.assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;

        // Check collection details.  If sized collection, size should be updated.
        let verified_collection_details = DEFAULT_COLLECTION_DETAILS.map(|details| match details {
            #[allow(deprecated)]
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            verified_collection_details
        );

        // Change the collection to have a different update authority.
        let new_collection_update_authority = Keypair::new();
        collection_parent_nft
            .change_update_authority(&mut context, new_collection_update_authority.pubkey())
            .await
            .unwrap();

        // Convert to DigitalAsset.
        let mut collection_parent_da = collection_parent_nft
            .into_digital_asset(&mut context, Some(parent_master_edition_account.pubkey))
            .await;

        // Burn collection parent.
        let args = BurnArgs::V1 { amount: 1 };
        let payer = context.payer.dirty_clone();
        collection_parent_da
            .burn(&mut context, payer, args, None, None)
            .await
            .unwrap();

        // Assert that metadata, edition, token and token record accounts are closed.
        collection_parent_da
            .assert_burned(&mut context)
            .await
            .unwrap();

        // Unverify using the new collection update authority.
        let args = VerificationArgs::CollectionV1;
        let err = da
            .unverify(
                &mut context,
                new_collection_update_authority,
                args,
                None,
                None,
                Some(collection_parent_da.mint.pubkey()),
                Some(collection_parent_da.metadata),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);

        da.assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;
    }

    #[tokio::test]
    async fn pass_unverify_burned_nft_parent_using_item_update_authority() {
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Check collection details.
        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            DEFAULT_COLLECTION_DETAILS
        );

        // Verify.
        let payer = context.payer.dirty_clone();
        let args = VerificationArgs::CollectionV1;
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

        let verified_collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: true,
        });

        da.assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;

        // Check collection details.  If sized collection, size should be updated.
        let verified_collection_details = DEFAULT_COLLECTION_DETAILS.map(|details| match details {
            #[allow(deprecated)]
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            verified_collection_details
        );

        // Change the collection to have a different update authority.
        let new_collection_update_authority = Keypair::new();
        collection_parent_nft
            .change_update_authority(&mut context, new_collection_update_authority.pubkey())
            .await
            .unwrap();

        // Convert to DigitalAsset.
        let mut collection_parent_da = collection_parent_nft
            .into_digital_asset(&mut context, Some(parent_master_edition_account.pubkey))
            .await;

        // Burn collection parent.
        let args = BurnArgs::V1 { amount: 1 };
        let payer = context.payer.dirty_clone();
        collection_parent_da
            .burn(&mut context, payer, args, None, None)
            .await
            .unwrap();

        // Assert that metadata, edition, token and token record accounts are closed.
        collection_parent_da
            .assert_burned(&mut context)
            .await
            .unwrap();

        // Unverify using item update authority.
        let payer = context.payer.dirty_clone();
        let args = VerificationArgs::CollectionV1;
        da.unverify(
            &mut context,
            payer,
            args,
            None,
            None,
            Some(collection_parent_da.mint.pubkey()),
            Some(collection_parent_da.metadata),
        )
        .await
        .unwrap();

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;
    }

    #[tokio::test]
    async fn burned_pnft_collections_update_authority_cannot_unverify() {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;

        // Change the collection to have a different update authority.
        let new_collection_update_authority = Keypair::new();
        airdrop(
            &mut context,
            &new_collection_update_authority.pubkey(),
            LAMPORTS_PER_SOL,
        )
        .await
        .unwrap();

        let mut args = UpdateArgs::default_as_update_authority();
        match &mut args {
            UpdateArgs::AsUpdateAuthorityV2 {
                new_update_authority,
                ..
            } => *new_update_authority = Some(new_collection_update_authority.pubkey()),
            _ => panic!("Unexpected enum variant"),
        }

        let payer = context.payer.dirty_clone();
        test_items
            .collection_parent_da
            .update(&mut context, payer, args)
            .await
            .unwrap();

        // Burn collection parent.
        let args = BurnArgs::V1 { amount: 1 };
        let payer = context.payer.dirty_clone();
        test_items
            .collection_parent_da
            .burn(&mut context, payer, args, None, None)
            .await
            .unwrap();

        // Assert that metadata, edition, token and token record accounts are closed.
        test_items
            .collection_parent_da
            .assert_burned(&mut context)
            .await
            .unwrap();

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let err = test_items
            .da
            .unverify(
                &mut context,
                new_collection_update_authority,
                args,
                None,
                None,
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);

        let verified_collection = test_items.collection.clone().map(|mut c| {
            c.verified = true;
            c
        });

        test_items
            .da
            .assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;
    }

    #[tokio::test]
    async fn pass_unverify_burned_pnft_parent_using_item_update_authority() {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;

        // Change the collection to have a different update authority.
        let new_collection_update_authority = Keypair::new();

        let mut args = UpdateArgs::default_as_update_authority();
        match &mut args {
            UpdateArgs::AsUpdateAuthorityV2 {
                new_update_authority,
                ..
            } => *new_update_authority = Some(new_collection_update_authority.pubkey()),
            _ => panic!("Unexpected enum variant"),
        }

        let payer = context.payer.dirty_clone();
        test_items
            .collection_parent_da
            .update(&mut context, payer, args)
            .await
            .unwrap();

        // Burn collection parent.
        let args = BurnArgs::V1 { amount: 1 };
        let payer = context.payer.dirty_clone();
        test_items
            .collection_parent_da
            .burn(&mut context, payer, args, None, None)
            .await
            .unwrap();

        // Assert that metadata, edition, token and token record accounts are closed.
        test_items
            .collection_parent_da
            .assert_burned(&mut context)
            .await
            .unwrap();

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        test_items
            .da
            .unverify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap();

        test_items
            .da
            .assert_item_collection_matches_on_chain(&mut context, &test_items.collection)
            .await;
    }

    #[tokio::test]
    async fn pass_unverify_burned_pnft_parent_using_item_collection_delegate() {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;

        // Burn collection parent.
        let args = BurnArgs::V1 { amount: 1 };
        let payer = context.payer.dirty_clone();
        test_items
            .collection_parent_da
            .burn(&mut context, payer, args, None, None)
            .await
            .unwrap();

        // Assert that metadata, edition, token and token record accounts are closed.
        test_items
            .collection_parent_da
            .assert_burned(&mut context)
            .await
            .unwrap();

        // Create a metadata update delegate for the item.
        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let payer = context.payer.dirty_clone();
        let payer_pubkey = payer.pubkey();
        let delegate_args = DelegateArgs::CollectionV1 {
            authorization_data: None,
        };
        test_items
            .da
            .delegate(&mut context, payer, delegate.pubkey(), delegate_args)
            .await
            .unwrap();

        // Find delegate record PDA.
        let (delegate_record, _) = find_metadata_delegate_record_account(
            &test_items.da.mint.pubkey(),
            MetadataDelegateRole::Collection,
            &payer_pubkey,
            &delegate.pubkey(),
        );

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        test_items
            .da
            .unverify(
                &mut context,
                delegate,
                args,
                None,
                Some(delegate_record),
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap();

        test_items
            .da
            .assert_item_collection_matches_on_chain(&mut context, &test_items.collection)
            .await;
    }

    #[tokio::test]
    async fn pass_unverify_burned_pnft_parent_using_item_collection_item_delegate() {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;

        // Burn collection parent.
        let args = BurnArgs::V1 { amount: 1 };
        let payer = context.payer.dirty_clone();
        test_items
            .collection_parent_da
            .burn(&mut context, payer, args, None, None)
            .await
            .unwrap();

        // Assert that metadata, edition, token and token record accounts are closed.
        test_items
            .collection_parent_da
            .assert_burned(&mut context)
            .await
            .unwrap();

        // Create a metadata update delegate for the item.
        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let payer = context.payer.dirty_clone();
        let payer_pubkey = payer.pubkey();
        let delegate_args = DelegateArgs::CollectionItemV1 {
            authorization_data: None,
        };
        test_items
            .da
            .delegate(&mut context, payer, delegate.pubkey(), delegate_args)
            .await
            .unwrap();

        // Find delegate record PDA.
        let (delegate_record, _) = find_metadata_delegate_record_account(
            &test_items.da.mint.pubkey(),
            MetadataDelegateRole::CollectionItem,
            &payer_pubkey,
            &delegate.pubkey(),
        );

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        test_items
            .da
            .unverify(
                &mut context,
                delegate,
                args,
                None,
                Some(delegate_record),
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap();

        test_items
            .da
            .assert_item_collection_matches_on_chain(&mut context, &test_items.collection)
            .await;
    }

    #[tokio::test]
    async fn collections_collection_delegate_cannot_unverify_burned_pnft_parent() {
        let delegate_args = DelegateArgs::CollectionV1 {
            authorization_data: None,
        };

        let delegate_role = MetadataDelegateRole::Collection;

        other_metadata_delegates_cannot_unverify_burned_pnft_parent(
            AssetToDelegate::CollectionParent,
            delegate_args,
            delegate_role,
        )
        .await;
    }

    #[tokio::test]
    async fn collections_collection_item_delegate_cannot_unverify_burned_pnft_parent() {
        let delegate_args = DelegateArgs::CollectionItemV1 {
            authorization_data: None,
        };

        let delegate_role = MetadataDelegateRole::CollectionItem;

        other_metadata_delegates_cannot_unverify_burned_pnft_parent(
            AssetToDelegate::CollectionParent,
            delegate_args,
            delegate_role,
        )
        .await;
    }

    #[tokio::test]
    async fn collections_prgm_config_delegate_cannot_unverify_burned_pnft_parent() {
        let delegate_args = DelegateArgs::ProgrammableConfigV1 {
            authorization_data: None,
        };

        let delegate_role = MetadataDelegateRole::ProgrammableConfig;

        other_metadata_delegates_cannot_unverify_burned_pnft_parent(
            AssetToDelegate::CollectionParent,
            delegate_args,
            delegate_role,
        )
        .await;
    }

    #[tokio::test]
    async fn items_prgm_config_delegate_cannot_unverify_burned_pnft_parent() {
        let delegate_args = DelegateArgs::ProgrammableConfigV1 {
            authorization_data: None,
        };

        let delegate_role = MetadataDelegateRole::ProgrammableConfig;

        other_metadata_delegates_cannot_unverify_burned_pnft_parent(
            AssetToDelegate::Item,
            delegate_args,
            delegate_role,
        )
        .await;
    }

    async fn other_metadata_delegates_cannot_unverify_burned_pnft_parent(
        asset_to_delegate: AssetToDelegate,
        delegate_args: DelegateArgs,
        delegate_role: MetadataDelegateRole,
    ) {
        let mut context = program_test().start_with_context().await;

        // Use pNFT for collection parent for this test.
        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;

        // Create a metadata delegate.
        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let asset = match asset_to_delegate {
            AssetToDelegate::CollectionParent => &mut test_items.collection_parent_da,
            AssetToDelegate::Item => &mut test_items.da,
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

        // Burn collection parent.
        let args = BurnArgs::V1 { amount: 1 };
        let payer = context.payer.dirty_clone();
        test_items
            .collection_parent_da
            .burn(&mut context, payer, args, None, None)
            .await
            .unwrap();

        // Assert that metadata, edition, token and token record accounts are closed.
        test_items
            .collection_parent_da
            .assert_burned(&mut context)
            .await
            .unwrap();

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let err = test_items
            .da
            .unverify(
                &mut context,
                delegate,
                args,
                None,
                Some(delegate_record),
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);

        let verified_collection = test_items.collection.clone().map(|mut c| {
            c.verified = true;
            c
        });

        test_items
            .da
            .assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;
    }

    #[tokio::test]
    async fn collections_utility_delegate_cannot_unverify_burned_pnft_parent() {
        let mut context = program_test().start_with_context().await;

        // Use pNFT for collection parent for this test.
        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;

        // Create a Utility delegate for collection parent.
        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let payer = context.payer.dirty_clone();
        let delegate_args = DelegateArgs::UtilityV1 {
            amount: 1,
            authorization_data: None,
        };
        test_items
            .collection_parent_da
            .delegate(&mut context, payer, delegate.pubkey(), delegate_args)
            .await
            .unwrap();

        // Find the token_record account for the Utility Delegate.
        let (token_record, _) = find_token_record_account(
            &test_items.collection_parent_da.mint.pubkey(),
            &test_items.collection_parent_da.token.unwrap(),
        );

        // Burn collection parent.  Note the delegate has to be used as the authority in this case.
        let args = BurnArgs::V1 { amount: 1 };
        test_items
            .collection_parent_da
            .burn(&mut context, delegate.dirty_clone(), args, None, None)
            .await
            .unwrap();

        // Assert that metadata, edition, token and token record accounts are closed.
        test_items
            .collection_parent_da
            .assert_burned(&mut context)
            .await
            .unwrap();

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let err = test_items
            .da
            .unverify(
                &mut context,
                delegate,
                args,
                None,
                Some(token_record),
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap_err();

        // In this case the token record will be closed so we expect IncorrectOwner.
        assert_custom_error!(err, MetadataError::IncorrectOwner);

        let verified_collection = test_items.collection.clone().map(|mut c| {
            c.verified = true;
            c
        });

        test_items
            .da
            .assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;
    }

    #[tokio::test]
    async fn items_utility_delegate_cannot_unverify_burned_pnft_parent() {
        let mut context = program_test().start_with_context().await;

        // Use pNFT for collection parent for this test.
        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;

        // Create a Utility delegate for the item.
        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let payer = context.payer.dirty_clone();
        let delegate_args = DelegateArgs::UtilityV1 {
            amount: 1,
            authorization_data: None,
        };
        test_items
            .da
            .delegate(&mut context, payer, delegate.pubkey(), delegate_args)
            .await
            .unwrap();

        // Find the token_record account for the Utility Delegate.
        let (token_record, _) =
            find_token_record_account(&test_items.da.mint.pubkey(), &test_items.da.token.unwrap());

        // Burn collection parent.
        let args = BurnArgs::V1 { amount: 1 };
        let payer = context.payer.dirty_clone();
        test_items
            .collection_parent_da
            .burn(&mut context, payer, args, None, None)
            .await
            .unwrap();

        // Assert that metadata, edition, token and token record accounts are closed.
        test_items
            .collection_parent_da
            .assert_burned(&mut context)
            .await
            .unwrap();

        // Unverify.
        let args = VerificationArgs::CollectionV1;
        let err = test_items
            .da
            .unverify(
                &mut context,
                delegate,
                args,
                None,
                Some(token_record),
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(test_items.collection_parent_da.metadata),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);

        let verified_collection = test_items.collection.clone().map(|mut c| {
            c.verified = true;
            c
        });

        test_items
            .da
            .assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;
    }

    struct CollectionTestItems {
        da: DigitalAsset,
        collection: Option<Collection>,
        collection_parent_da: DigitalAsset,
    }

    async fn create_mint_verify_collection_check(
        context: &mut ProgramTestContext,
        collection_details: Option<CollectionDetails>,
        item_token_standard: TokenStandard,
        collection_token_standard: TokenStandard,
    ) -> CollectionTestItems {
        // Create a collection parent NFT or pNFT with the CollectionDetails struct populated.
        let mut collection_parent_da = DigitalAsset::new();
        collection_parent_da
            .create_and_mint_collection_parent(
                context,
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
            context,
            item_token_standard,
            None,
            None,
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        assert_collection_unverified_item_and_parent(
            context,
            &da,
            &collection,
            &collection_parent_da,
            &collection_details,
        )
        .await;

        // Verify.
        let args = VerificationArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        da.verify(
            context,
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

        assert_collection_verified_item_and_parent(
            context,
            &da,
            &collection,
            &collection_parent_da,
            &collection_details,
        )
        .await;

        CollectionTestItems {
            da,
            collection,
            collection_parent_da,
        }
    }

    async fn assert_collection_unverified_item_and_parent(
        context: &mut ProgramTestContext,
        item_da: &DigitalAsset,
        collection: &Option<Collection>,
        collection_parent_da: &DigitalAsset,
        collection_details: &Option<CollectionDetails>,
    ) {
        item_da
            .assert_item_collection_matches_on_chain(context, collection)
            .await;

        collection_parent_da
            .assert_collection_details_matches_on_chain(context, collection_details)
            .await;
    }

    async fn assert_collection_verified_item_and_parent(
        context: &mut ProgramTestContext,
        item_da: &DigitalAsset,
        collection: &Option<Collection>,
        collection_parent_da: &DigitalAsset,
        collection_details: &Option<CollectionDetails>,
    ) {
        let verified_collection = collection.clone().map(|mut c| {
            c.verified = true;
            c
        });

        item_da
            .assert_item_collection_matches_on_chain(context, &verified_collection)
            .await;

        // Collection size should be updated.
        let verified_collection_details = collection_details.clone().map(|details| match details {
            #[allow(deprecated)]
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        collection_parent_da
            .assert_collection_details_matches_on_chain(context, &verified_collection_details)
            .await;
    }
}
