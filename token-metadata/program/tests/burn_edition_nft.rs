#![cfg(feature = "test-bpf")]
pub mod utils;

use num_traits::FromPrimitive;
use solana_program_test::*;
use solana_sdk::{instruction::InstructionError, signer::Signer, transaction::TransactionError};
use utils::*;

mod burn_edition_nft {
    use mpl_token_metadata::{
        error::MetadataError,
        state::{MasterEditionV2 as ProgramMasterEdition, TokenMetadataAccount},
    };
    use solana_program::pubkey::Pubkey;
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

    #[tokio::test]
    pub async fn fail_to_burn_master_edition() {
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

        let second_nft = Metadata::new();
        second_nft
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

        let second_master_edition = MasterEditionV2::new(&second_nft);
        second_master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        let err = burn_edition(
            &mut context,
            original_nft.pubkey,
            &payer,
            second_nft.mint.pubkey(),
            original_nft.mint.pubkey(),
            second_nft.token.pubkey(),
            master_edition.pubkey,
            second_nft.pubkey,
            Pubkey::new_unique(), // throwaway key since it will fail before it gets to this check
        )
        .await
        .unwrap_err();

        assert_custom_error!(err, MetadataError::NotAPrintEdition);
    }

    #[tokio::test]
    pub async fn no_master_edition() {
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

        let second_print_edition = EditionMarker::new(&original_nft, &master_edition, 2);
        second_print_edition.create(&mut context).await.unwrap();

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        let err = burn_edition(
            &mut context,
            original_nft.pubkey,
            &payer,
            print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            print_edition.token.pubkey(),
            second_print_edition.pubkey,
            print_edition.pubkey,
            Pubkey::new_unique(), // throwaway key since it will fail before it gets to this check
        )
        .await
        .unwrap_err();

        assert_custom_error!(err, MetadataError::NotAMasterEdition);
    }

    #[tokio::test]
    pub async fn invalid_edition_marker() {
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

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        let err = burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &payer,
            print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            print_edition.token.pubkey(),
            master_edition.pubkey,
            print_edition.new_edition_pubkey,
            Pubkey::new_unique(),
        )
        .await
        .unwrap_err();

        // The error will be IncorrectOwner since the random pubkey we generated is not a PDA owned
        // by the token metadata program.
        assert_custom_error!(err, MetadataError::IncorrectOwner);

        // Create a second print edition to try to pass off as the edition marker. It's owned by token metadata
        // so will pass that check but will fail with IncorrectEditonMarker.

        let second_print_edition = EditionMarker::new(&original_nft, &master_edition, 2);
        second_print_edition.create(&mut context).await.unwrap();

        let err = burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &payer,
            print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            print_edition.token.pubkey(),
            master_edition.pubkey,
            print_edition.new_edition_pubkey,
            second_print_edition.new_edition_pubkey,
        )
        .await
        .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidEditionMarker);
    }

    #[tokio::test]
    pub async fn master_supply_is_decremented() {
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

        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap()
            .unwrap();

        let master_edition_struct: ProgramMasterEdition =
            ProgramMasterEdition::safe_deserialize(&master_edition_account.data).unwrap();

        assert!(master_edition_struct.supply == 1);

        let second_print_edition = EditionMarker::new(&original_nft, &master_edition, 2);
        second_print_edition.create(&mut context).await.unwrap();

        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap()
            .unwrap();

        let master_edition_struct: ProgramMasterEdition =
            ProgramMasterEdition::safe_deserialize(&master_edition_account.data).unwrap();

        assert!(master_edition_struct.supply == 2);

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

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

        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap()
            .unwrap();

        let master_edition_struct: ProgramMasterEdition =
            ProgramMasterEdition::safe_deserialize(&master_edition_account.data).unwrap();

        assert!(master_edition_struct.supply == 1);

        burn_edition(
            &mut context,
            second_print_edition.new_metadata_pubkey,
            &payer,
            second_print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            second_print_edition.token.pubkey(),
            master_edition.pubkey,
            second_print_edition.new_edition_pubkey,
            second_print_edition.pubkey,
        )
        .await
        .unwrap();

        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap()
            .unwrap();

        let master_edition_struct: ProgramMasterEdition =
            ProgramMasterEdition::safe_deserialize(&master_edition_account.data).unwrap();

        assert!(master_edition_struct.supply == 0);
    }

    #[tokio::test]
    pub async fn edition_mask_changed_correctly() {
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

        let second_print_edition = EditionMarker::new(&original_nft, &master_edition, 2);
        second_print_edition.create(&mut context).await.unwrap();

        let third_print_edition = EditionMarker::new(&original_nft, &master_edition, 3);
        third_print_edition.create(&mut context).await.unwrap();

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        let edition_marker_account = context
            .banks_client
            .get_account(second_print_edition.pubkey)
            .await
            .unwrap()
            .unwrap();

        // Ledger is the 31 bytes after the key.
        let ledger = &edition_marker_account.data[1..];

        assert!(ledger[0] == 0b0111_0000);

        burn_edition(
            &mut context,
            second_print_edition.new_metadata_pubkey,
            &payer,
            second_print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            second_print_edition.token.pubkey(),
            master_edition.pubkey,
            second_print_edition.new_edition_pubkey,
            second_print_edition.pubkey,
        )
        .await
        .unwrap();

        let edition_marker_account = context
            .banks_client
            .get_account(second_print_edition.pubkey)
            .await
            .unwrap()
            .unwrap();

        // Ledger is the 31 bytes after the key.
        let ledger = &edition_marker_account.data[1..];

        assert!(ledger[0] == 0b0101_0000);
    }
}
