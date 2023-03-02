#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_metadata::{
    error::MetadataError,
    instruction::{
        builders::VerifyBuilder, DelegateArgs, InstructionBuilder, MetadataDelegateRole, VerifyArgs,
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

// Note: at the time these tests were created, the only Metadata delegates that have been
// implemented are `Collection`, `Update`, and `ProgrammableConfig`.  We have tested each of these
// cases.

// Also at this time, a collection parent NFT cannot have a token standard of
// `ProgrammableNonFungible`.  This means that using the new delegate handler, the only Token
// delegate that can be issued for a collection parent NFT is `TokenDelegateRole::Standard`, which
// means no token record PDA account will be created. Thus, we cannot properly test that the
// Standard delegate is not authorized to verify a collection because if we send a non-existent
// token record account to the verify handler (as a delegate record), it simply fails because the
// owner is incorrect.

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
        let args = VerifyArgs::CreatorV1;
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
        let args = VerifyArgs::CreatorV1;
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
        let args = VerifyArgs::CreatorV1;
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
        let args = VerifyArgs::CreatorV1;

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
        // See `standard_delegate_fails`.
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Verify.
        let args = VerifyArgs::CollectionV1;
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Verify.
        let args = VerifyArgs::CollectionV1;
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Verify.
        let args = VerifyArgs::CollectionV1;
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Verify.
        let args = VerifyArgs::CollectionV1;
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Verify.
        let args = VerifyArgs::CollectionV1;
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Verify.
        let args = VerifyArgs::CollectionV1;
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Verify.
        let args = VerifyArgs::CollectionV1;
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
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

        // Check collection details.  Should be default.
        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            DEFAULT_COLLECTION_DETAILS
        );

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

        // Check collection details.  Should still be default.
        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            DEFAULT_COLLECTION_DETAILS
        );

        // Verify.
        let args = VerifyArgs::CollectionV1;
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

        let verified_collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: true,
        });

        da.assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;

        // Check collection details.  Size should be updated.
        let verified_collection_details = DEFAULT_COLLECTION_DETAILS.map(|details| match details {
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            verified_collection_details
        );

        // Skip ahead.
        context.warp_to_slot(2).unwrap();

        // Verify again.
        let args = VerifyArgs::CollectionV1;
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

        da.assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;

        // Check collection details.  It should not have updated this time.
        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            verified_collection_details
        );
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Verify.
        let args = VerifyArgs::CollectionV1;
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Verify.
        let args = VerifyArgs::CollectionV1;
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Verify.
        let args = VerifyArgs::CollectionV1;
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
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
        let args = VerifyArgs::CollectionV1;
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;
    }

    #[tokio::test]
    async fn pass_item_nft_sized_collection_update_authority_collection_and_item_old_handlers() {
        pass_item_nft_collection_update_authority_collection_and_item_old_handlers(
            DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_item_nft_unsized_collection_update_authority_collection_and_item_old_handlers() {
        pass_item_nft_collection_update_authority_collection_and_item_old_handlers(None).await;
    }

    async fn pass_item_nft_collection_update_authority_collection_and_item_old_handlers(
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

        let collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        });

        // Create our item using old handler.
        let name = "Test".to_string();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();
        let test_metadata = Metadata::new();
        test_metadata
            .create_v2(
                &mut context,
                name,
                symbol,
                uri,
                None,
                10,
                false,
                collection,
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
            .build(VerifyArgs::CollectionV1)
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
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            verified_collection_details
        );
    }

    #[tokio::test]
    async fn pass_item_pnft_sized_collection_update_authority_collection_old_handler() {
        pass_item_pnft_collection_update_authority_collection_old_handler(
            DEFAULT_COLLECTION_DETAILS,
        )
        .await;
    }

    #[tokio::test]
    async fn pass_item_pnft_unsized_collection_update_authority_collection_old_handler() {
        pass_item_pnft_collection_update_authority_collection_old_handler(None).await;
    }

    async fn pass_item_pnft_collection_update_authority_collection_old_handler(
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

        // Check collection details.
        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(collection_metadata.collection_details, collection_details);

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
        assert_eq!(collection_metadata.collection_details, collection_details);

        // Verify.
        let args = VerifyArgs::CollectionV1;
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

        let verified_collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: true,
        });

        da.assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;

        // Check collection details.  If sized collection, size should be updated.
        let verified_collection_details = collection_details.map(|details| match details {
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            verified_collection_details
        );
    }

    #[tokio::test]
    async fn pass_with_changed_collection_update_authority() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
        let (collection_parent_nft, parent_master_edition_account) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        // Check collection details.
        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            DEFAULT_COLLECTION_DETAILS
        );

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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Check collection details.
        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            DEFAULT_COLLECTION_DETAILS
        );

        // Verify using the new collection update authority.
        let args = VerifyArgs::CollectionV1;
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

        let verified_collection = Some(Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: true,
        });

        da.assert_item_collection_matches_on_chain(&mut context, &verified_collection)
            .await;

        // Check collection details.  If sized collection, size should be updated.
        let verified_collection_details = DEFAULT_COLLECTION_DETAILS.map(|details| match details {
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            verified_collection_details
        );
    }

    #[tokio::test]
    async fn item_update_authority_cannot_verify() {
        let mut context = program_test().start_with_context().await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
        let (collection_parent_nft, parent_master_edition_account) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        // Check collection details.
        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            DEFAULT_COLLECTION_DETAILS
        );

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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Check collection details.
        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            DEFAULT_COLLECTION_DETAILS
        );

        // Verify using the new collection update authority.
        let payer = context.payer.dirty_clone();
        let args = VerifyArgs::CollectionV1;
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

        da.assert_item_collection_matches_on_chain(&mut context, &collection)
            .await;

        // Check collection details.  It should not be updated.
        let collection_metadata = collection_parent_nft.get_data(&mut context).await;
        assert_eq!(
            collection_metadata.collection_details,
            DEFAULT_COLLECTION_DETAILS
        );
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

        // Create a collection parent NFT with the CollectionDetails struct populated.
        let mut collection_parent_da = DigitalAsset::new();
        collection_parent_da
            .create_and_mint_collection_parent(
                &mut context,
                TokenStandard::NonFungible,
                None,
                None,
                1,
                collection_details.clone(),
            )
            .await
            .unwrap();

        collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &collection_details)
            .await;

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
        let args = VerifyArgs::CollectionV1;
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
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &verified_collection_details)
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

        // Create a collection parent NFT with the CollectionDetails struct populated.
        let mut collection_parent_da = DigitalAsset::new();
        collection_parent_da
            .create_and_mint_collection_parent(
                &mut context,
                TokenStandard::NonFungible,
                None,
                None,
                1,
                collection_details.clone(),
            )
            .await
            .unwrap();

        collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &collection_details)
            .await;

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
        let args = VerifyArgs::CollectionV1;
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
            CollectionDetails::V1 { size } => CollectionDetails::V1 { size: size + 1 },
        });

        collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &verified_collection_details)
            .await;
    }

    #[tokio::test]
    async fn update_delegate_cannot_verify() {
        let delegate_args = DelegateArgs::UpdateV1 {
            authorization_data: None,
        };

        let delegate_role = MetadataDelegateRole::Update;

        other_metadata_delegates_cannot_verify(delegate_args, delegate_role).await;
    }

    #[tokio::test]
    async fn programmable_config_delegate_cannot_verify() {
        let delegate_args = DelegateArgs::ProgrammableConfigV1 {
            authorization_data: None,
        };

        let delegate_role = MetadataDelegateRole::ProgrammableConfig;

        other_metadata_delegates_cannot_verify(delegate_args, delegate_role).await;
    }

    async fn other_metadata_delegates_cannot_verify(
        delegate_args: DelegateArgs,
        delegate_role: MetadataDelegateRole,
    ) {
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

        collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &DEFAULT_COLLECTION_DETAILS)
            .await;

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

        let payer = context.payer.dirty_clone();
        let payer_pubkey = payer.pubkey();
        collection_parent_da
            .delegate(&mut context, payer, delegate.pubkey(), delegate_args)
            .await
            .unwrap();

        // Find delegate record PDA.
        let (delegate_record, _) = find_metadata_delegate_record_account(
            &collection_parent_da.mint.pubkey(),
            delegate_role,
            &payer_pubkey,
            &delegate.pubkey(),
        );

        // Verify.
        let args = VerifyArgs::CollectionV1;
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

        // Create a collection parent NFT with the CollectionDetails struct populated.
        let mut first_collection_parent_da = DigitalAsset::new();
        first_collection_parent_da
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

        first_collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &DEFAULT_COLLECTION_DETAILS)
            .await;

        // Create a collection parent NFT with the CollectionDetails struct populated.
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
        let args = VerifyArgs::CollectionV1;
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
    async fn standard_delegate_fails() {
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

        collection_parent_da
            .assert_collection_details_matches_on_chain(&mut context, &DEFAULT_COLLECTION_DETAILS)
            .await;

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
        let args = VerifyArgs::CollectionV1;
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
}
