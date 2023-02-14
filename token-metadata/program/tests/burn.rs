#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_metadata::{
    error::MetadataError,
    instruction::{builders::BurnBuilder, BurnArgs, InstructionBuilder},
    state::{Creator, Key, Metadata as ProgramMetadata, TokenStandard},
};
use num_traits::FromPrimitive;
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction, TransactionError},
};
use spl_token::state::Account as TokenAccount;
use utils::*;

mod success_cases {
    use super::*;

    #[tokio::test]
    async fn burn_nonfungible() {
        let mut context = program_test().start_with_context().await;

        let mut da = DigitalAsset::new();
        da.create_and_mint(&mut context, TokenStandard::NonFungible, None, None, 1)
            .await
            .unwrap();

        let args = BurnArgs::V1 { amount: 1 };

        da.burn(&mut context, args, None, None).await.unwrap();

        // Assert that metadata, edition and token account are closed.
        da.assert_burned(&mut context).await.unwrap();
    }

    #[tokio::test]
    async fn burn_programmable_nonfungible() {
        let mut context = program_test().start_with_context().await;

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
        )
        .await
        .unwrap();

        let args = BurnArgs::V1 { amount: 1 };

        da.burn(&mut context, args, None, None).await.unwrap();

        // Assert that metadata, edition and token account are closed.
        da.assert_burned(&mut context).await.unwrap();
    }

    #[tokio::test]
    async fn burn_nonfungible_edition() {
        let mut context = program_test().start_with_context().await;

        let nft = Metadata::new();
        let nft_master_edition = MasterEditionV2::new(&nft);
        let nft_edition_marker = EditionMarker::new(&nft, &nft_master_edition, 1);

        let payer_key = context.payer.pubkey();

        nft.create(
            &mut context,
            "Test".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            Some(vec![Creator {
                address: payer_key,
                verified: true,
                share: 100,
            }]),
            10,
            false,
            0,
        )
        .await
        .unwrap();

        nft_master_edition
            .create(&mut context, Some(10))
            .await
            .unwrap();

        nft_edition_marker.create(&mut context).await.unwrap();

        let edition_marker = nft_edition_marker.get_data(&mut context).await;
        let print_edition = get_account(&mut context, &nft_edition_marker.new_edition_pubkey).await;

        assert_eq!(edition_marker.ledger[0], 64);
        assert_eq!(edition_marker.key, Key::EditionMarker);
        assert_eq!(print_edition.data[0], 1);

        let args = BurnArgs::V1 { amount: 1 };

        let mut builder = BurnBuilder::new();
        builder
            .authority(context.payer.pubkey())
            .metadata(nft_edition_marker.new_metadata_pubkey)
            .edition(nft_edition_marker.new_edition_pubkey)
            .mint(nft_edition_marker.mint.pubkey())
            .token(nft_edition_marker.token.pubkey())
            .parent_mint(nft.mint.pubkey())
            .parent_token(nft.token.pubkey())
            .parent_edition(nft_master_edition.pubkey)
            .edition_marker(nft_edition_marker.pubkey);

        let burn_ix = builder.build(args).unwrap().instruction();

        let transaction = Transaction::new_signed_with_payer(
            &[burn_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();

        // Metadata, and token account are burned.
        let print_md = context
            .banks_client
            .get_account(nft_edition_marker.new_metadata_pubkey)
            .await
            .unwrap();
        let token_account = context
            .banks_client
            .get_account(nft_edition_marker.token.pubkey())
            .await
            .unwrap();
        let print_edition_account = context
            .banks_client
            .get_account(nft_edition_marker.new_edition_pubkey)
            .await
            .unwrap();

        assert!(print_md.is_none());
        assert!(token_account.is_none());
        assert!(print_edition_account.is_none());
    }

    #[tokio::test]
    async fn burn_fungible() {
        let mut context = program_test().start_with_context().await;

        let initial_amount = 10;
        let burn_amount = 1;

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::Fungible,
            None,
            None,
            initial_amount,
        )
        .await
        .unwrap();

        let args = BurnArgs::V1 {
            amount: burn_amount,
        };

        da.burn(&mut context, args, None, None).await.unwrap();

        // We only burned one token, so the token account should still exist.
        let token_account = context
            .banks_client
            .get_account(da.token.unwrap())
            .await
            .unwrap()
            .unwrap();

        let token = TokenAccount::unpack(&token_account.data).unwrap();

        assert_eq!(token.amount, initial_amount - burn_amount);

        let burn_remaining = initial_amount - burn_amount;

        let args = BurnArgs::V1 {
            amount: burn_remaining,
        };

        da.burn(&mut context, args, None, None).await.unwrap();

        // The token account should be closed now.
        let token_account = context
            .banks_client
            .get_account(da.token.unwrap())
            .await
            .unwrap();

        assert!(token_account.is_none());
    }
}

mod failure_cases {
    use mpl_token_metadata::state::{
        Collection, CollectionDetails, PrintSupply, TokenMetadataAccount,
    };

    use super::*;

    #[tokio::test]
    async fn fail_to_burn_master_edition_with_existing_prints() {
        let mut context = program_test().start_with_context().await;

        let mut original_nft = DigitalAsset::new();
        original_nft
            .create_and_mint_nonfungible(&mut context, PrintSupply::Limited(10))
            .await
            .unwrap();

        let print_nft = original_nft.print_edition(&mut context, 1).await.unwrap();

        // Metadata, Print Edition and token account exist.
        let md_account = context
            .banks_client
            .get_account(print_nft.metadata)
            .await
            .unwrap();
        let token_account = context
            .banks_client
            .get_account(print_nft.token.unwrap())
            .await
            .unwrap();
        let print_edition_account = context
            .banks_client
            .get_account(print_nft.edition.unwrap())
            .await
            .unwrap();

        assert!(md_account.is_some());
        assert!(token_account.is_some());
        assert!(print_edition_account.is_some());

        let err = original_nft
            .burn(&mut context, BurnArgs::V1 { amount: 1 }, None, None)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::MasterEditionHasPrints);
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
            ProgramMetadata::safe_deserialize(&mut parent_nft_account.data.as_slice()).unwrap();

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
            ProgramMetadata::safe_deserialize(&mut parent_nft_account.data.as_slice()).unwrap();

        if let Some(details) = parent_metadata.collection_details {
            match details {
                CollectionDetails::V1 { size } => {
                    assert_eq!(size, 1);
                }
            }
        } else {
            panic!("CollectionDetails is not set");
        }

        let mut da: DigitalAsset = collection_item_nft
            .into_digital_asset(&mut context, Some(item_master_edition_account.pubkey))
            .await;

        // Burn the NFT w/o passing in collection metadata. This should fail.
        let err = da
            .burn(&mut context, BurnArgs::V1 { amount: 1 }, None, None)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::MissingCollectionMetadata);
    }

    #[tokio::test]
    async fn burn_unsized_collection_item() {
        let mut context = program_test().start_with_context().await;

        // Create a Collection Parent NFT without the CollectionDetails struct
        let collection_parent_nft = Metadata::new();
        collection_parent_nft
            .create_v3_default(&mut context)
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
                Some(collection),
                None,
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

        let mut da: DigitalAsset = collection_item_nft
            .into_digital_asset(&mut context, Some(item_master_edition_account.pubkey))
            .await;

        // Burn the NFT
        da.burn(
            &mut context,
            BurnArgs::V1 { amount: 1 },
            None,
            Some(collection_parent_nft.pubkey),
        )
        .await
        .unwrap();
    }
}
