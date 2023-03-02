#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_metadata::{
    error::MetadataError,
    instruction::{DelegateArgs, MetadataDelegateRole, VerifyArgs},
    pda::find_metadata_delegate_record_account,
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

mod verify_creator {
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
        let args = VerifyArgs::CreatorV1;
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
        let args = VerifyArgs::CreatorV1;
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
        let args = VerifyArgs::CreatorV1;
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
        let args = VerifyArgs::CreatorV1;

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
        let args = VerifyArgs::CreatorV1;

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

mod verify_collection {
    use super::*;

    #[tokio::test]
    async fn metadata_wrong_owner() {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;

        // Unverify.
        let args = VerifyArgs::CollectionV1;
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

        let verified_collection = Some(Collection {
            key: test_items.collection_parent_da.mint.pubkey(),
            verified: true,
        });

        test_items
            .da
            .assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;

        // Check collection details.  If sized collection, size should be updated.
        let verified_collection_details = DEFAULT_COLLECTION_DETAILS.map(|details| match details {
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        test_items
            .collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &verified_collection_details)
            .await;
    }

    #[tokio::test]
    async fn collection_mint_info_wrong_owner() {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;

        // Unverify.
        let args = VerifyArgs::CollectionV1;
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

        let verified_collection = Some(Collection {
            key: test_items.collection_parent_da.mint.pubkey(),
            verified: true,
        });

        test_items
            .da
            .assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;

        // Check collection details.  If sized collection, size should be updated.
        let verified_collection_details = DEFAULT_COLLECTION_DETAILS.map(|details| match details {
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        test_items
            .collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &verified_collection_details)
            .await;
    }

    #[tokio::test]
    async fn collection_metadata_info_wrong_owner() {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;

        // Unverify.
        let args = VerifyArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        let collection_metadata_info_wrong_owner = Keypair::new().pubkey();
        let err = test_items
            .da
            .unverify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(test_items.collection_parent_da.mint.pubkey()),
                Some(collection_metadata_info_wrong_owner),
            )
            .await
            .unwrap_err();

        // In this case it will be MintMismatch because it fails a derivation check.
        assert_custom_error!(err, MetadataError::MintMismatch);

        let verified_collection = Some(Collection {
            key: test_items.collection_parent_da.mint.pubkey(),
            verified: true,
        });

        test_items
            .da
            .assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;

        // Check collection details.  If sized collection, size should be updated.
        let verified_collection_details = DEFAULT_COLLECTION_DETAILS.map(|details| match details {
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        test_items
            .collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &verified_collection_details)
            .await;
    }

    #[tokio::test]
    async fn missing_collection_mint_info() {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;

        // Unverify.
        let args = VerifyArgs::CollectionV1;
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

        let verified_collection = Some(Collection {
            key: test_items.collection_parent_da.mint.pubkey(),
            verified: true,
        });

        test_items
            .da
            .assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;

        // Check collection details.  If sized collection, size should be updated.
        let verified_collection_details = DEFAULT_COLLECTION_DETAILS.map(|details| match details {
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        test_items
            .collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &verified_collection_details)
            .await;
    }

    #[tokio::test]
    async fn missing_collection_metadata_info() {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;

        // Unverify.
        let args = VerifyArgs::CollectionV1;
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

        let verified_collection = Some(Collection {
            key: test_items.collection_parent_da.mint.pubkey(),
            verified: true,
        });

        test_items
            .da
            .assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;

        // Check collection details.  If sized collection, size should be updated.
        let verified_collection_details = DEFAULT_COLLECTION_DETAILS.map(|details| match details {
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        test_items
            .collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &verified_collection_details)
            .await;
    }

    #[tokio::test]
    async fn pass_item_pnft_sized_collection_update_authority_collection_new_handler() {
        pass_collection_update_authority_collection_new_handler(
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_item_nft_sized_collection_update_authority_collection_new_handler() {
        pass_collection_update_authority_collection_new_handler(
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::NonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_item_pnft_unsized_collection_update_authority_collection_new_handler() {
        pass_collection_update_authority_collection_new_handler(
            None,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_item_nft_unsized_collection_update_authority_collection_new_handler() {
        pass_collection_update_authority_collection_new_handler(None, TokenStandard::NonFungible)
            .await;
    }

    async fn pass_collection_update_authority_collection_new_handler(
        collection_details: Option<CollectionDetails>,
        item_token_standard: TokenStandard,
    ) {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            collection_details.clone(),
            item_token_standard,
        )
        .await;

        // Unverify.
        let args = VerifyArgs::CollectionV1;
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
            .assert_item_collection_matches_on_chain(
                &mut context,
                &test_items.unverified_collection,
            )
            .await;

        test_items
            .collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &collection_details)
            .await;
    }

    #[tokio::test]
    async fn pass_item_pnft_delegated_authority_sized_collection_new_handler() {
        pass_delegated_authority_collection_new_handler(
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_item_nft_delegated_authority_sized_collection_new_handler() {
        pass_delegated_authority_collection_new_handler(
            DEFAULT_COLLECTION_DETAILS,
            TokenStandard::NonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_item_pnft_delegated_authority_unsized_collection_new_handler() {
        pass_delegated_authority_collection_new_handler(
            None,
            TokenStandard::ProgrammableNonFungible,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_item_nft_delegated_authority_unsized_collection_new_handler() {
        pass_delegated_authority_collection_new_handler(None, TokenStandard::NonFungible).await;
    }

    async fn pass_delegated_authority_collection_new_handler(
        collection_details: Option<CollectionDetails>,
        item_token_standard: TokenStandard,
    ) {
        let mut context = program_test().start_with_context().await;

        let mut test_items = create_mint_verify_collection_check(
            &mut context,
            collection_details.clone(),
            item_token_standard,
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
        let args = VerifyArgs::CollectionV1;
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
            .assert_item_collection_matches_on_chain(
                &mut context,
                &test_items.unverified_collection,
            )
            .await;

        test_items
            .collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &collection_details)
            .await;
    }

    struct CollectionTestItems {
        collection_parent_da: DigitalAsset,
        unverified_collection: Option<Collection>,
        da: DigitalAsset,
    }

    async fn create_mint_verify_collection_check(
        context: &mut ProgramTestContext,
        collection_details: Option<CollectionDetails>,
        item_token_standard: TokenStandard,
    ) -> CollectionTestItems {
        // Create a collection parent NFT with the CollectionDetails struct populated.
        let mut collection_parent_da = DigitalAsset::new();
        collection_parent_da
            .create_and_mint_collection_parent(
                context,
                TokenStandard::NonFungible,
                None,
                None,
                1,
                collection_details.clone(),
            )
            .await
            .unwrap();

        collection_parent_da
            .assert_collection_details_matches_on_chain(context, &collection_details)
            .await;

        // Create and mint item.
        let unverified_collection = Some(Collection {
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
            unverified_collection.clone(),
        )
        .await
        .unwrap();

        da.assert_item_collection_matches_on_chain(context, &unverified_collection)
            .await;

        collection_parent_da
            .assert_collection_details_matches_on_chain(context, &collection_details)
            .await;

        // Verify.
        let args = VerifyArgs::CollectionV1;
        let payer = context.payer.dirty_clone();
        da.verify(
            context,
            payer,
            args,
            None,
            None,
            Some(collection_parent_da.mint.pubkey()),
            Some(collection_parent_da.metadata),
            Some(collection_parent_da.master_edition.unwrap()),
        )
        .await
        .unwrap();

        let verified_collection = Some(Collection {
            key: collection_parent_da.mint.pubkey(),
            verified: true,
        });

        da.assert_item_collection_matches_on_chain(context, &verified_collection)
            .await;

        // Check collection details.  If sized collection, size should be updated.
        let verified_collection_details = collection_details.map(|details| match details {
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        collection_parent_da
            .assert_collection_details_matches_on_chain(context, &verified_collection_details)
            .await;

        CollectionTestItems {
            collection_parent_da,
            unverified_collection,
            da,
        }
    }
}
