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
            original_nft.token.pubkey(),
            master_edition.pubkey,
            print_edition.new_edition_pubkey,
            print_edition.pubkey,
        )
        .await
        .unwrap();

        // Metadata, edition, and token account are burned.
        let print_md = context
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

        // Token Metadata accounts may still be open because they are no longer being re-assigned
        // to the system program immediately, but if they exist they should have a
        // data length of 1 (just the disciriminator byte, set to Uninitialized).

        if let Some(account) = print_md {
            assert_eq!(account.data.len(), 1);
        }

        assert!(token_account.is_none());
        assert!(print_edition_account.is_none());

        // Edition marker account should also be burned, because that was the only print edition on it.
        let edition_marker_account = context
            .banks_client
            .get_account(print_edition.pubkey)
            .await
            .unwrap();
        if let Some(account) = edition_marker_account {
            assert_eq!(account.data.len(), 0);
        }

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
        original_nft.create_v3_default(&mut context).await.unwrap();

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
            original_nft.token.pubkey(),
            master_edition.pubkey,
            print_edition.new_edition_pubkey,
            print_edition.pubkey,
        )
        .await
        .unwrap_err();

        // We've passed in the correct token account associated with the old owner but
        // it has 0 tokens so we get this error.
        assert_custom_error!(err, MetadataError::InsufficientTokenBalance);

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
            original_nft.token.pubkey(),
            master_edition.pubkey,
            print_edition.new_edition_pubkey,
            print_edition.pubkey,
        )
        .await
        .unwrap_err();

        // We've passed in the correct token account associated with the new owner but
        // the old owner is not the current owner of the account so this should fail with
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
            original_nft.token.pubkey(),
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
            original_nft.token.pubkey(),
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
        original_nft.create_v3_default(&mut context).await.unwrap();

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
            original_nft.token.pubkey(),
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
        original_nft.create_v3_default(&mut context).await.unwrap();

        let second_nft = Metadata::new();
        second_nft.create_v3_default(&mut context).await.unwrap();

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
            original_nft.token.pubkey(),
            master_edition.pubkey,
            second_nft.pubkey,
            // it will fail before it evaluates edition marker but we need an account that will pass initial owner checks
            original_nft.pubkey,
        )
        .await
        .unwrap_err();

        assert_custom_error!(err, MetadataError::MintMismatch);
    }

    #[tokio::test]
    pub async fn no_master_edition() {
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
            original_nft.token.pubkey(),
            second_print_edition.pubkey,
            print_edition.pubkey,
            // Use the second print edition as the master edition, which will pass the
            // initial owner checks but fail to match the mint.
            print_edition.pubkey,
        )
        .await
        .unwrap_err();

        assert_custom_error!(err, MetadataError::MintMismatch);
    }

    #[tokio::test]
    async fn invalid_master_edition() {
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

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        let err = burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &payer,
            print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            print_edition.token.pubkey(),
            original_nft.token.pubkey(),
            // Use a key that will pass the owner check but is not a master edition.
            print_edition.new_edition_pubkey,
            print_edition.new_edition_pubkey,
            print_edition.pubkey,
        )
        .await
        .unwrap_err();

        // The random pubkey will have a data len of zero so is not a Master Edition.
        assert_custom_error!(err, MetadataError::NotAMasterEdition);

        // Create a second master edition to try to pass off as the correct one. It's owned by token metadata
        // and has the right len of data, so will pass that check but will fail with InvalidMasterEdition because
        // it's derivation is incorrect.

        let new_nft = Metadata::new();
        new_nft.create_v3_default(&mut context).await.unwrap();

        let incorrect_master_edition = MasterEditionV2::new(&new_nft);
        incorrect_master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let err = burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &payer,
            print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            print_edition.token.pubkey(),
            original_nft.token.pubkey(),
            incorrect_master_edition.pubkey,
            print_edition.new_edition_pubkey,
            print_edition.pubkey,
        )
        .await
        .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidMasterEdition);
    }

    #[tokio::test]
    pub async fn invalid_print_edition() {
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

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        let err = burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &payer,
            print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            print_edition.token.pubkey(),
            original_nft.token.pubkey(),
            master_edition.pubkey,
            // Use a key that will pass the owner check but is not a print edition.
            master_edition.pubkey,
            print_edition.pubkey,
        )
        .await
        .unwrap_err();

        // The random pubkey will have a data len of zero so is not a Print Edition.
        assert_custom_error!(err, MetadataError::NotAPrintEdition);

        // Create a second print edition to try to pass off as the correct one. It's owned by token metadata
        // and has the right data length, so will pass those checks, but will fail with InvalidPrintEdition
        // because the derivation will be incorrect.

        let second_print_edition = EditionMarker::new(&original_nft, &master_edition, 2);
        second_print_edition.create(&mut context).await.unwrap();

        let err = burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &payer,
            print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            print_edition.token.pubkey(),
            original_nft.token.pubkey(),
            master_edition.pubkey,
            second_print_edition.new_edition_pubkey,
            print_edition.new_edition_pubkey,
        )
        .await
        .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidPrintEdition);
    }

    #[tokio::test]
    pub async fn invalid_edition_marker() {
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

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        let err = burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &payer,
            print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            print_edition.token.pubkey(),
            original_nft.token.pubkey(),
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
        // so will pass that check but will fail with IncorrectEditionMarker.

        let second_print_edition = EditionMarker::new(&original_nft, &master_edition, 2);
        second_print_edition.create(&mut context).await.unwrap();

        let err = burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &payer,
            print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            print_edition.token.pubkey(),
            original_nft.token.pubkey(),
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
        original_nft.create_v3_default(&mut context).await.unwrap();

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
        assert!(master_edition_struct.max_supply == Some(10));

        let mut second_print_edition = EditionMarker::new(&original_nft, &master_edition, 2);
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

        // Transfer second edition to a different owner.
        let user = Keypair::new();
        airdrop(&mut context, &user.pubkey(), 1_000_000_000)
            .await
            .unwrap();

        context.warp_to_slot(10).unwrap();

        second_print_edition
            .transfer(&mut context, &user.pubkey())
            .await
            .unwrap();
        let new_owner_token_account =
            get_associated_token_address(&user.pubkey(), &second_print_edition.mint.pubkey());

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Master edition owner burning.
        burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &payer,
            print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            print_edition.token.pubkey(),
            original_nft.token.pubkey(),
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

        // Master edition owner burning should decrement the supply.
        assert!(master_edition_struct.supply == 1);
        assert!(master_edition_struct.max_supply == Some(10));

        // Second owner burning.
        burn_edition(
            &mut context,
            second_print_edition.new_metadata_pubkey,
            &user,
            second_print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            new_owner_token_account,
            original_nft.token.pubkey(),
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

        // Second owner burning should decrement the supply.
        assert!(master_edition_struct.supply == 0);
    }

    #[tokio::test]
    pub async fn edition_mask_changed_correctly() {
        let mut context = program_test().start_with_context().await;

        let original_nft = Metadata::new();
        original_nft.create_v3_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&original_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        context.warp_to_slot(10).unwrap();

        let (print_editions, _end_slot) = master_edition
            .mint_editions(&mut context, &original_nft, 10, 10)
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

        assert!(ledger[0] == 0b0111_1111);
        assert!(ledger[1] == 0b1110_0000);

        // Burn the second one
        burn_edition(
            &mut context,
            print_editions[1].new_metadata_pubkey,
            &payer,
            print_editions[1].mint.pubkey(),
            original_nft.mint.pubkey(),
            print_editions[1].token.pubkey(),
            original_nft.token.pubkey(),
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

        // One bit flipped here
        assert!(ledger[0] == 0b0101_1111);
        // None here
        assert!(ledger[1] == 0b1110_0000);

        // Burn the last one
        burn_edition(
            &mut context,
            print_editions[9].new_metadata_pubkey,
            &payer,
            print_editions[9].mint.pubkey(),
            original_nft.mint.pubkey(),
            print_editions[9].token.pubkey(),
            original_nft.token.pubkey(),
            master_edition.pubkey,
            print_editions[9].new_edition_pubkey,
            print_editions[9].pubkey,
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

        // Stays the same
        assert!(ledger[0] == 0b0101_1111);
        // One bit flipped
        assert!(ledger[1] == 0b1100_0000);
    }

    #[tokio::test]
    pub async fn reprint_burned_edition() {
        // Reprinting a burned edition should work when the owner is the same for
        // the master edition and print edition. Otherwise, it should fail.
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

        // Print a new edition and transfer to a user.
        let mut user_print_edition = EditionMarker::new(&original_nft, &master_edition, 2);
        user_print_edition.create(&mut context).await.unwrap();

        let user = Keypair::new();
        airdrop(&mut context, &user.pubkey(), 1_000_000_000)
            .await
            .unwrap();

        context.warp_to_slot(10).unwrap();

        user_print_edition
            .transfer(&mut context, &user.pubkey())
            .await
            .unwrap();
        let new_owner_token_account =
            get_associated_token_address(&user.pubkey(), &user_print_edition.mint.pubkey());

        // Metadata, Print Edition and token account exist.
        assert!(print_edition.exists_on_chain(&mut context).await);
        assert!(user_print_edition.exists_on_chain(&mut context).await);

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Burn owner's edition.
        burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &payer,
            print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            print_edition.token.pubkey(),
            original_nft.token.pubkey(),
            master_edition.pubkey,
            print_edition.new_edition_pubkey,
            print_edition.pubkey,
        )
        .await
        .unwrap();

        // Burn user's edition.
        burn_edition(
            &mut context,
            user_print_edition.new_metadata_pubkey,
            &user,
            user_print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            new_owner_token_account,
            original_nft.token.pubkey(),
            master_edition.pubkey,
            user_print_edition.new_edition_pubkey,
            user_print_edition.pubkey,
        )
        .await
        .unwrap();

        // Metadata, Print Edition and token account do not exist.
        assert!(!print_edition.exists_on_chain(&mut context).await);
        assert!(!user_print_edition.exists_on_chain(&mut context).await);

        // Reprint owner's burned edition
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        // Metadata, Print Edition and token account exist.
        assert!(print_edition.exists_on_chain(&mut context).await);

        // Reprint user's burned edition: this should fail.
        let user_print_edition = EditionMarker::new(&original_nft, &master_edition, 2);
        let err = user_print_edition.create(&mut context).await.unwrap_err();

        assert_custom_error!(err, MetadataError::AlreadyInitialized);
    }

    #[tokio::test]
    async fn cannot_modify_wrong_master_edition() {
        let mut context = program_test().start_with_context().await;

        // Someone else's NFT
        let other_nft = Metadata::new();
        other_nft.create_v3_default(&mut context).await.unwrap();

        let other_master_edition = MasterEditionV2::new(&other_nft);
        other_master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let new_update_authority = Keypair::new();
        other_nft
            .change_update_authority(&mut context, new_update_authority.pubkey())
            .await
            .unwrap();

        let other_print_edition = EditionMarker::new(&other_nft, &other_master_edition, 1);
        other_print_edition.create(&mut context).await.unwrap();

        let our_nft = Metadata::new();
        our_nft.create_v3_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&our_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let print_edition = EditionMarker::new(&our_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // We pass in our edition NFT and someone else's master edition and try to modify their supply.
        let err = burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &payer,
            print_edition.mint.pubkey(),
            other_nft.mint.pubkey(),
            print_edition.token.pubkey(),
            other_nft.token.pubkey(),
            other_master_edition.pubkey,
            print_edition.new_edition_pubkey,
            other_print_edition.pubkey,
        )
        .await
        .unwrap_err();

        assert_custom_error!(err, MetadataError::PrintEditionDoesNotMatchMasterEdition);
    }

    #[tokio::test]
    async fn mint_mismatches() {
        let mut context = program_test().start_with_context().await;

        let nft = Metadata::new();
        nft.create_v3_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let print_edition = EditionMarker::new(&nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();
        let second_print_edition = EditionMarker::new(&nft, &master_edition, 2);
        second_print_edition.create(&mut context).await.unwrap();

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Wrong print edition mint account.
        let err = burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &payer,
            second_print_edition.mint.pubkey(),
            nft.mint.pubkey(),
            print_edition.token.pubkey(),
            nft.token.pubkey(),
            master_edition.pubkey,
            print_edition.new_edition_pubkey,
            print_edition.pubkey,
        )
        .await
        .unwrap_err();

        assert_custom_error!(err, MetadataError::MintMismatch);

        // Wrong master edition mint account.

        let other_nft = Metadata::new();
        other_nft.create_v3_default(&mut context).await.unwrap();

        let other_master_edition = MasterEditionV2::new(&other_nft);
        other_master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        // Wrong master edition mint account.
        let err = burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &payer,
            print_edition.mint.pubkey(),
            other_nft.mint.pubkey(), // wrong
            print_edition.token.pubkey(),
            nft.token.pubkey(),
            master_edition.pubkey,
            print_edition.new_edition_pubkey,
            print_edition.pubkey,
        )
        .await
        .unwrap_err();

        assert_custom_error!(err, MetadataError::MintMismatch);
    }
}
