#![cfg(feature = "test-bpf")]
pub mod utils;

use borsh::BorshSerialize;
use mpl_token_metadata::{
    error::MetadataError,
    instruction,
    state::{Collection, Creator, Key, MAX_MASTER_EDITION_LEN},
    ID,
};
use num_traits::FromPrimitive;
use solana_program_test::*;
use solana_sdk::{
    account::AccountSharedData,
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
};
use utils::*;

// NOTE: these tests depend on the token-vault program having been compiled
// via (cd ../../token-vault/program/ && cargo build-bpf)
mod mint_new_edition_from_master_edition_via_token {

    use solana_program::native_token::LAMPORTS_PER_SOL;

    use super::*;
    #[tokio::test]
    async fn success() {
        let mut context = program_test().start_with_context().await;
        let payer_key = context.payer.pubkey();
        let test_metadata = Metadata::new();
        let test_master_edition = MasterEditionV2::new(&test_metadata);
        let test_edition_marker = EditionMarker::new(&test_metadata, &test_master_edition, 1);

        test_metadata
            .create_v3(
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
                None,
                None,
                None,
            )
            .await
            .unwrap();

        test_master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        test_edition_marker.create(&mut context).await.unwrap();

        let edition_marker = test_edition_marker.get_data(&mut context).await;

        assert_eq!(edition_marker.ledger[0], 64);
        assert_eq!(edition_marker.key, Key::EditionMarker);
    }

    #[tokio::test]
    async fn success_v2() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        let creator = Keypair::new();

        let creator_pub = creator.pubkey();
        airdrop(&mut context, &creator_pub.clone(), 3 * LAMPORTS_PER_SOL)
            .await
            .unwrap();
        let test_master_edition = MasterEditionV2::new(&test_metadata);
        let test_collection = Metadata::new();
        test_collection
            .create_v3_default(&mut context)
            .await
            .unwrap();
        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();
        test_metadata
            .create_v3(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                Some(vec![Creator {
                    address: creator_pub,
                    verified: false,
                    share: 100,
                }]),
                10,
                false,
                Some(Collection {
                    key: test_collection.mint.pubkey(),
                    verified: false,
                }),
                None,
                None,
            )
            .await
            .unwrap();

        test_master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let tx = Transaction::new_signed_with_payer(
            [instruction::sign_metadata(
                mpl_token_metadata::ID,
                test_metadata.pubkey,
                creator_pub,
            )]
            .as_ref(),
            Some(&creator_pub),
            &[&creator],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        let kpbytes = &context.payer;
        let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();
        test_metadata
            .verify_collection(
                &mut context,
                test_collection.pubkey,
                &kp,
                test_collection.mint.pubkey(),
                collection_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap();
        let test_edition_marker = EditionMarker::new(&test_metadata, &test_master_edition, 1);
        test_edition_marker.create(&mut context).await.unwrap();

        let edition_marker = test_edition_marker.get_data(&mut context).await;

        assert_eq!(edition_marker.ledger[0], 64);
        assert_eq!(edition_marker.key, Key::EditionMarker);
    }

    #[tokio::test]
    async fn fail_invalid_token_program() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        let test_master_edition = MasterEditionV2::new(&test_metadata);
        let test_edition_marker = EditionMarker::new(&test_metadata, &test_master_edition, 1);

        test_metadata.create_v3_default(&mut context).await.unwrap();

        test_master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let result = test_edition_marker
            .create_with_invalid_token_program(&mut context)
            .await
            .unwrap_err();
        assert_custom_error!(result, MetadataError::InvalidTokenProgram);
    }

    #[tokio::test]
    async fn fail_invalid_mint() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        let test_master_edition = MasterEditionV2::new(&test_metadata);
        let test_edition_marker = EditionMarker::new(&test_metadata, &test_master_edition, 1);
        let fake_mint = Keypair::new();
        let fake_account = Keypair::new();
        let payer_pubkey = context.payer.pubkey();

        test_metadata.create_v3_default(&mut context).await.unwrap();

        test_master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        create_mint(&mut context, &fake_mint, &payer_pubkey, None, 0)
            .await
            .unwrap();

        create_token_account(
            &mut context,
            &fake_account,
            &fake_mint.pubkey(),
            &payer_pubkey,
        )
        .await
        .unwrap();

        mint_tokens(
            &mut context,
            &fake_mint.pubkey(),
            &fake_account.pubkey(),
            1,
            &payer_pubkey,
            None,
        )
        .await
        .unwrap();

        let tx = Transaction::new_signed_with_payer(
            &[instruction::mint_new_edition_from_master_edition_via_token(
                ID,
                test_edition_marker.new_metadata_pubkey,
                test_edition_marker.new_edition_pubkey,
                test_edition_marker.master_edition_pubkey,
                fake_mint.pubkey(),
                context.payer.pubkey(),
                context.payer.pubkey(),
                context.payer.pubkey(),
                fake_account.pubkey(),
                context.payer.pubkey(),
                test_edition_marker.metadata_pubkey,
                test_edition_marker.metadata_mint_pubkey,
                test_edition_marker.edition,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, &context.payer, &context.payer],
            context.last_blockhash,
        );

        let result = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        assert_custom_error!(result, MetadataError::TokenAccountMintMismatchV2);
    }

    #[tokio::test]
    async fn fail_edition_already_initialized() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        let test_master_edition = MasterEditionV2::new(&test_metadata);
        let test_edition_marker = EditionMarker::new(&test_metadata, &test_master_edition, 1);
        let test_edition_marker1 = EditionMarker::new(&test_metadata, &test_master_edition, 1);

        test_metadata.create_v3_default(&mut context).await.unwrap();

        test_master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        test_edition_marker.create(&mut context).await.unwrap();
        let result = test_edition_marker1.create(&mut context).await.unwrap_err();
        assert_custom_error!(result, MetadataError::AlreadyInitialized);
    }

    #[tokio::test]
    async fn fail_to_mint_edition_override_0() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        let test_master_edition = MasterEditionV2::new(&test_metadata);
        let test_edition_marker = EditionMarker::new(&test_metadata, &test_master_edition, 0);

        test_metadata.create_v3_default(&mut context).await.unwrap();

        test_master_edition
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let result = test_edition_marker.create(&mut context).await.unwrap_err();
        assert_custom_error!(result, MetadataError::EditionOverrideCannotBeZero);
    }

    #[tokio::test]
    async fn fail_to_mint_edition_num_zero() {
        // Make sure we can't mint 0th edition from a Master Edition with a max supply > 0.
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        let test_master_edition = MasterEditionV2::new(&test_metadata);
        let test_edition_marker = EditionMarker::new(&test_metadata, &test_master_edition, 0);

        test_metadata.create_v3_default(&mut context).await.unwrap();

        test_master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let result = test_edition_marker.create(&mut context).await.unwrap_err();
        assert_custom_error!(result, MetadataError::EditionOverrideCannotBeZero);
    }

    #[tokio::test]
    async fn increment_master_edition_supply() {
        let mut context = program_test().start_with_context().await;

        let original_nft = Metadata::new();
        original_nft.create_v3_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&original_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        // Metadata, Print Edition and token account exist.
        assert!(print_edition.exists_on_chain(&mut context).await);

        let master_edition_struct = master_edition.get_data(&mut context).await;

        // We've printed one edition and our max supply is 10.
        assert!(master_edition_struct.supply == 1);
        assert!(master_edition_struct.max_supply == Some(10));

        // Mint edition number 5 and supply should go up to 2.
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 5);
        print_edition.create(&mut context).await.unwrap();

        let master_edition_struct = master_edition.get_data(&mut context).await;

        assert!(master_edition_struct.supply == 2);
        assert!(master_edition_struct.max_supply == Some(10));

        // Mint edition number 4 and supply should go up to 3.
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 4);
        print_edition.create(&mut context).await.unwrap();

        let mut master_edition_struct = master_edition.get_data(&mut context).await;
        let mut master_edition_account = get_account(&mut context, &master_edition.pubkey).await;

        assert!(master_edition_struct.supply == 3);
        assert!(master_edition_struct.max_supply == Some(10));

        // Simulate a collection where there are are missing editions with numbers lower than the current
        // supply value and ensure they can still be minted.
        master_edition_struct.supply = 8;
        let mut data = master_edition_struct.try_to_vec().unwrap();
        let filler = vec![0u8; MAX_MASTER_EDITION_LEN - data.len()];
        data.extend_from_slice(&filler[..]);
        master_edition_account.data = data;

        let master_edition_shared_data: AccountSharedData = master_edition_account.into();
        context.set_account(&master_edition.pubkey, &master_edition_shared_data);

        assert!(master_edition_struct.supply == 8);
        assert!(master_edition_struct.max_supply == Some(10));

        // Mint edition number 2, this will succeed but supply will incremement.
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 2);
        print_edition.create(&mut context).await.unwrap();

        let master_edition_struct = master_edition.get_data(&mut context).await;

        assert!(master_edition_struct.supply == 9);
        assert!(master_edition_struct.max_supply == Some(10));

        // Mint edition number 10 and supply should increase by 1 to 10.
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 10);
        print_edition.create(&mut context).await.unwrap();

        let master_edition_struct = master_edition.get_data(&mut context).await;

        assert!(master_edition_struct.supply == 10);
        assert!(master_edition_struct.max_supply == Some(10));

        // Mint another edition and it should succeed, but supply should stay the same since it's already reached max supply.
        // This allows minting missing editions even when the supply has erroneously reached
        // the max supply, since the bit mask is the source of truth for which particular editions have been minted.
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 6);
        print_edition.create(&mut context).await.unwrap();

        let master_edition_struct = master_edition.get_data(&mut context).await;

        assert!(master_edition_struct.supply == 10);
        assert!(master_edition_struct.max_supply == Some(10));
    }

    #[tokio::test]
    async fn cannot_mint_edition_num_higher_than_max_supply() {
        let mut context = program_test().start_with_context().await;

        let original_nft = Metadata::new();
        original_nft.create_v3_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&original_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        // Mint the first print edition.
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        let master_edition_struct = master_edition.get_data(&mut context).await;
        assert!(master_edition_struct.supply == 1);
        assert!(master_edition_struct.max_supply == Some(10));

        // Try mint edition number 11, this should fail.
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 11);
        let err = print_edition.create(&mut context).await.unwrap_err();

        assert_custom_error!(err, MetadataError::EditionNumberGreaterThanMaxSupply);

        // Try mint edition number 999, this should fail.
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 999);
        let err = print_edition.create(&mut context).await.unwrap_err();

        assert_custom_error!(err, MetadataError::EditionNumberGreaterThanMaxSupply);
    }

    #[tokio::test]
    async fn cannot_remint_existing_edition() {
        let mut context = program_test().start_with_context().await;

        let original_nft = Metadata::new();
        original_nft.create_v3_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&original_nft);
        master_edition
            .create_v3(&mut context, Some(999))
            .await
            .unwrap();

        // Mint a couple non-sequential editions.
        let edition_1 = EditionMarker::new(&original_nft, &master_edition, 1);
        edition_1.create(&mut context).await.unwrap();
        let edition_99 = EditionMarker::new(&original_nft, &master_edition, 99);
        edition_99.create(&mut context).await.unwrap();

        let master_edition_struct = master_edition.get_data(&mut context).await;
        assert!(master_edition_struct.supply == 2);
        assert!(master_edition_struct.max_supply == Some(999));

        // Try to remint edition numbers 1 and 99, this should fail.
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        let err = print_edition.create(&mut context).await.unwrap_err();

        assert_custom_error!(err, MetadataError::AlreadyInitialized);

        let print_edition = EditionMarker::new(&original_nft, &master_edition, 99);
        let err = print_edition.create(&mut context).await.unwrap_err();

        assert_custom_error!(err, MetadataError::AlreadyInitialized);
    }

    #[tokio::test]
    async fn can_mint_out_missing_editions() {
        // Editions with the older override logic could have missing editions even though supply == max_supply.
        // This test ensures that the new logic can mint out missing editions even when supply == max_supply.
        let mut context = program_test().start_with_context().await;

        let original_nft = Metadata::new();
        original_nft.create_v3_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&original_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        // Start with a supply of 10. Mint out edition number 10 and then artificially set the supply to 10
        // to simulate the old edition override logic.
        let edition_10 = EditionMarker::new(&original_nft, &master_edition, 10);
        edition_10.create(&mut context).await.unwrap();

        let mut master_edition_struct = master_edition.get_data(&mut context).await;
        let mut master_edition_account = get_account(&mut context, &master_edition.pubkey).await;

        master_edition_struct.supply = 10;
        let mut data = master_edition_struct.try_to_vec().unwrap();
        let filler = vec![0u8; MAX_MASTER_EDITION_LEN - data.len()];
        data.extend_from_slice(&filler[..]);
        master_edition_account.data = data;

        let master_edition_shared_data: AccountSharedData = master_edition_account.into();
        context.set_account(&master_edition.pubkey, &master_edition_shared_data);

        assert!(master_edition_struct.supply == 10);
        assert!(master_edition_struct.max_supply == Some(10));

        // Try to mint edition number 11, this should fail.
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 11);
        let err = print_edition.create(&mut context).await.unwrap_err();

        assert_custom_error!(err, MetadataError::EditionNumberGreaterThanMaxSupply);

        // We should be able to mint out missing editions 1-9.
        for i in 1..10 {
            let print_edition = EditionMarker::new(&original_nft, &master_edition, i);
            print_edition.create(&mut context).await.unwrap();
        }

        let master_edition_struct = master_edition.get_data(&mut context).await;

        // Supply should still be 10.
        assert!(master_edition_struct.supply == 10);
        assert!(master_edition_struct.max_supply == Some(10));
    }
}
