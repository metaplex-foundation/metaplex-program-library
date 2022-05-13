#![cfg(feature = "test-bpf")]
pub mod utils;

use solana_program_test::*;
use solana_sdk::signature::Signer;
use utils::*;
mod burn_nft {

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
}
