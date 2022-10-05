#![cfg(feature = "test-bpf")]

pub mod utils;

use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::Transaction};
use utils::*;

mod trifle {
    use mpl_token_metadata::state::{CollectionDetails, Creator, EscrowAuthority};
    use mpl_trifle::{
        instruction::{
            add_collection_constraint_to_escrow_constraint_model,
            add_none_constraint_to_escrow_constraint_model,
            add_tokens_constraint_to_escrow_constraint_model,
            create_escrow_constraint_model_account, create_trifle_account, transfer_in,
            transfer_out,
        },
        pda::{find_escrow_constraint_model_address, find_trifle_address},
        state::{
            escrow_constraints::EscrowConstraintModel, fuse_options::FuseOptions, trifle::Trifle,
        },
    };
    use solana_program::borsh::try_from_slice_unchecked;

    use super::*;

    #[tokio::test]
    async fn test_create_trifle_account() {
        let mut context = program_test().start_with_context().await;

        let metadata = Metadata::new();
        let master_edition = MasterEditionV2::new(&metadata);
        let payer_pubkey = context.payer.pubkey();
        metadata
            .create_v2(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                Some(vec![Creator {
                    address: payer_pubkey,
                    verified: true,
                    share: 100,
                }]),
                10,
                true,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        master_edition
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let test_collection = Metadata::new();
        test_collection
            .create_v3(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "".to_string(),
                None,
                0,
                false,
                None,
                None,
                None,
                Some(CollectionDetails::V1 { size: 1 }),
            )
            .await
            .unwrap();

        let (escrow_constraint_model_addr, _) =
            find_escrow_constraint_model_address(&context.payer.pubkey(), "Test");

        let create_constraint_model_ix = create_escrow_constraint_model_account(
            &mpl_trifle::id(),
            &escrow_constraint_model_addr,
            &context.payer.pubkey(),
            &context.payer.pubkey(),
            "Test".to_string(),
            None,
            FuseOptions::new().with_burn(true).with_track(true),
        );

        let add_none_constraint_ix = add_none_constraint_to_escrow_constraint_model(
            &mpl_trifle::id(),
            &escrow_constraint_model_addr,
            &context.payer.pubkey(),
            &context.payer.pubkey(),
            "test".to_string(),
            0,
        );

        let add_collection_constraint_ix = add_collection_constraint_to_escrow_constraint_model(
            &mpl_trifle::id(),
            &escrow_constraint_model_addr,
            &context.payer.pubkey(),
            &context.payer.pubkey(),
            &test_collection.mint.pubkey(),
            &test_collection.pubkey,
            "collection".to_string(),
            0,
        );

        let add_tokens_constraint_ix = add_tokens_constraint_to_escrow_constraint_model(
            &mpl_trifle::id(),
            &escrow_constraint_model_addr,
            &context.payer.pubkey(),
            &context.payer.pubkey(),
            "tokens".to_string(),
            0,
            vec![metadata.mint.pubkey()],
        );

        let tx = Transaction::new_signed_with_payer(
            &[
                create_constraint_model_ix,
                add_none_constraint_ix,
                add_tokens_constraint_ix,
                add_collection_constraint_ix,
            ],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let (trifle_addr, _) = find_trifle_address(
            &metadata.mint.pubkey(),
            &context.payer.pubkey(),
            &escrow_constraint_model_addr,
        );

        let (escrow_addr, _) = mpl_token_metadata::escrow::pda::find_escrow_account(
            &metadata.mint.pubkey(),
            &EscrowAuthority::Creator(trifle_addr.to_owned()),
        );

        let create_trifle_account_ix = create_trifle_account(
            &mpl_trifle::id(),
            &escrow_addr,
            &metadata.pubkey,
            &metadata.mint.pubkey(),
            &metadata.token.pubkey(),
            &master_edition.pubkey,
            &trifle_addr,
            &context.payer.pubkey(),
            &escrow_constraint_model_addr,
            &context.payer.pubkey(),
        );

        let tx = Transaction::new_signed_with_payer(
            &[create_trifle_account_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let trifle_account = context
            .banks_client
            .get_account(trifle_addr)
            .await
            .unwrap()
            .unwrap();

        let trifle_account_data: Trifle = try_from_slice_unchecked(&trifle_account.data).unwrap();
        println!("trifle_account: {:#?}", trifle_account_data);
        let constraint_account = context
            .banks_client
            .get_account(escrow_constraint_model_addr)
            .await
            .unwrap()
            .unwrap();
        let constraint_account_data: EscrowConstraintModel =
            try_from_slice_unchecked(&constraint_account.data).unwrap();
        println!("constraint_account: {:#?}", constraint_account_data);

        // Build the attribute
        let attribute_metadata = Metadata::new();
        let attribute_master_edition = MasterEditionV2::new(&attribute_metadata);
        attribute_metadata
            .create_v2(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                true,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        attribute_master_edition
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let trifle_attribute_token_account =
            spl_associated_token_account::get_associated_token_address(
                &escrow_addr,
                &attribute_metadata.mint.pubkey(),
            );

        let transfer_in_ix = transfer_in(
            mpl_trifle::id(),
            trifle_addr,
            escrow_constraint_model_addr,
            escrow_addr,
            context.payer.pubkey(),
            context.payer.pubkey(),
            attribute_metadata.mint.pubkey(),
            attribute_metadata.token.pubkey(),
            trifle_attribute_token_account,
            attribute_metadata.pubkey,
            metadata.mint.pubkey(),
            metadata.token.pubkey(),
            Some(attribute_master_edition.pubkey),
            None,
            "test".to_string(),
            1,
        );

        let transfer_in_tx = Transaction::new_signed_with_payer(
            &[transfer_in_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(transfer_in_tx)
            .await
            .unwrap();

        let trifle_account = context
            .banks_client
            .get_account(trifle_addr)
            .await
            .unwrap()
            .unwrap();

        let trifle_account_data: Trifle = try_from_slice_unchecked(&trifle_account.data).unwrap();
        println!("trifle_account: {:#?}", trifle_account_data);
        let constraint_account = context
            .banks_client
            .get_account(escrow_constraint_model_addr)
            .await
            .unwrap()
            .unwrap();
        let constraint_account_data: EscrowConstraintModel =
            try_from_slice_unchecked(&constraint_account.data).unwrap();
        println!("constraint_account: {:#?}", constraint_account_data);

        let metadata_attribute_token_account =
            spl_associated_token_account::get_associated_token_address(
                &context.payer.pubkey(),
                &attribute_metadata.mint.pubkey(),
            );

        let transfer_out_ix = transfer_out(
            mpl_trifle::id(),
            trifle_addr,
            escrow_constraint_model_addr,
            escrow_addr,
            context.payer.pubkey(),
            context.payer.pubkey(),
            attribute_metadata.mint.pubkey(),
            trifle_attribute_token_account,
            metadata_attribute_token_account,
            attribute_metadata.pubkey,
            metadata.mint.pubkey(),
            metadata.token.pubkey(),
            "test".to_string(),
            1,
        );

        let transfer_out_tx = Transaction::new_signed_with_payer(
            &[transfer_out_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(transfer_out_tx)
            .await
            .unwrap();

        let trifle_account = context
            .banks_client
            .get_account(trifle_addr)
            .await
            .unwrap()
            .unwrap();

        let trifle_account_data: Trifle = try_from_slice_unchecked(&trifle_account.data).unwrap();
        println!("trifle_account: {:#?}", trifle_account_data);
        let constraint_account = context
            .banks_client
            .get_account(escrow_constraint_model_addr)
            .await
            .unwrap()
            .unwrap();
        let constraint_account_data: EscrowConstraintModel =
            try_from_slice_unchecked(&constraint_account.data).unwrap();
        println!("constraint_account: {:#?}", constraint_account_data);
    }
}
