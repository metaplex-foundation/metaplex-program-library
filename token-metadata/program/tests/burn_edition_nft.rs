#![cfg(feature = "test-bpf")]
pub mod utils;

use num_traits::FromPrimitive;
use solana_program_test::*;
use solana_sdk::{instruction::InstructionError, signer::Signer, transaction::TransactionError};
use utils::*;

mod burn_edition_nft {
    use mpl_token_metadata::error::MetadataError;
    use solana_sdk::signature::Keypair;

    use super::*;

    #[tokio::test]
    async fn successfully_burn_edition_nft() {
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

        context.warp_to_slot(10).unwrap();

        burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &payer,
            print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            print_edition.token.pubkey(),
            master_edition.pubkey,
            print_edition.new_edition_pubkey,
            print_edition.pubkey,
        )
        .await
        .unwrap();

        // Metadata, Edition and token account are burned.
        let md_account = context
            .banks_client
            .get_account(print_edition.new_metadata_pubkey)
            .await
            .unwrap();
        let edition_account = context
            .banks_client
            .get_account(print_edition.new_edition_pubkey)
            .await
            .unwrap();
        let token_account = context
            .banks_client
            .get_account(print_edition.token.pubkey())
            .await
            .unwrap();
        assert!(md_account.is_none());
        assert!(edition_account.is_none());
        assert!(token_account.is_none());

        // Edition marker account should also be burned, because that was the only print edition on it.
        let edition_marker_account = context
            .banks_client
            .get_account(print_edition.pubkey)
            .await
            .unwrap();
        assert!(edition_marker_account.is_none());

        // Master Edition on original NFT still exists.
        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap();
        assert!(master_edition_account.is_some());
    }

    #[tokio::test]
    async fn only_owner_can_burn() {
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

        let not_owner = Keypair::new();
        airdrop(&mut context, &not_owner.pubkey(), 1_000_000_000)
            .await
            .unwrap();

        let err = burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &not_owner,
            print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            print_edition.token.pubkey(),
            master_edition.pubkey,
            print_edition.new_edition_pubkey,
            print_edition.pubkey,
        )
        .await
        .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidOwner);
    }

    #[tokio::test]
    async fn update_authority_cannot_burn() {
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

        // NFT is created with context payer as the update authority so we need to update this before
        // creating the print edition, so it gets a copy of this new update authority.
        let new_update_authority = Keypair::new();

        original_nft
            .change_update_authority(&mut context, new_update_authority.pubkey())
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

        let err = burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &new_update_authority,
            print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            print_edition.token.pubkey(),
            master_edition.pubkey,
            print_edition.new_edition_pubkey,
            print_edition.pubkey,
        )
        .await
        .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidOwner);
    }
}
