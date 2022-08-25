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
    use spl_associated_token_account::get_associated_token_address;

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
        assert!(print_edition.exists_on_chain(&mut context).await);

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
        assert!(!print_edition.exists_on_chain(&mut context).await);

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
    async fn burn_edition_nft_in_separate_wallet() {
        // Burn a print edition that is in a separate wallet, so owned by a different account
        // than the master edition nft.
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
        let mut print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        // Transfer to new owner.
        let new_owner = Keypair::new();
        let new_owner_pubkey = new_owner.pubkey();
        airdrop(&mut context, &new_owner_pubkey, 1_000_000_000)
            .await
            .unwrap();

        context.warp_to_slot(10).unwrap();

        print_edition
            .transfer(&mut context, &new_owner_pubkey)
            .await
            .unwrap();

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Old owner should not be able to burn.
        let err = burn_edition(
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
        .unwrap_err();

        // We've passed in the correct token account associated with the old owner but
        // it has 0 tokens so we get this error.
        assert_custom_error!(err, MetadataError::NotEnoughTokens);

        // Old owner should not be able to burn even if we pass in the new token
        // account associated with the new owner.
        let new_owner_token_account =
            get_associated_token_address(&new_owner_pubkey, &print_edition.mint.pubkey());

        let err = burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &payer,
            print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            new_owner_token_account,
            master_edition.pubkey,
            print_edition.new_edition_pubkey,
            print_edition.pubkey,
        )
        .await
        .unwrap_err();

        // We've passed in the correct token account associated with the new owner but
        // the old owner is not the current owner of the account so this shuld fail with
        // InvalidOwner error.
        assert_custom_error!(err, MetadataError::InvalidOwner);

        // New owner can burn.
        burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &new_owner,
            print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            new_owner_token_account,
            master_edition.pubkey,
            print_edition.new_edition_pubkey,
            print_edition.pubkey,
        )
        .await
        .unwrap();

        // Metadata, Edition and token account are burned.
        assert!(!print_edition.exists_on_chain(&mut context).await);
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
        assert!(print_edition.exists_on_chain(&mut context).await);

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
        assert!(print_edition.exists_on_chain(&mut context).await);

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

        let print_editions = master_edition
            .mint_editions(&mut context, &original_nft, 3)
            .await
            .unwrap();

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        let edition_marker_account = context
            .banks_client
            .get_account(print_editions[1].pubkey)
            .await
            .unwrap()
            .unwrap();

        // Ledger is the 31 bytes after the key.
        let ledger = &edition_marker_account.data[1..];

        assert!(ledger[0] == 0b0111_0000);

        burn_edition(
            &mut context,
            print_editions[1].new_metadata_pubkey,
            &payer,
            print_editions[1].mint.pubkey(),
            original_nft.mint.pubkey(),
            print_editions[1].token.pubkey(),
            master_edition.pubkey,
            print_editions[1].new_edition_pubkey,
            print_editions[1].pubkey,
        )
        .await
        .unwrap();

        let edition_marker_account = context
            .banks_client
            .get_account(print_editions[1].pubkey)
            .await
            .unwrap()
            .unwrap();

        // Ledger is the 31 bytes after the key.
        let ledger = &edition_marker_account.data[1..];

        assert!(ledger[0] == 0b0101_0000);
    }

    #[tokio::test]
    pub async fn reprint_burned_edition() {
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
        assert!(print_edition.exists_on_chain(&mut context).await);

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

        // Metadata, Print Edition and token account do not exist.
        assert!(!print_edition.exists_on_chain(&mut context).await);

        // Reprint burned edition
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        // Metadata, Print Edition and token account exist.
        assert!(print_edition.exists_on_chain(&mut context).await);
    }
}
