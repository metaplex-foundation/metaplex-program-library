#![cfg(feature = "test-bpf")]

pub mod utils;

use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::Transaction};
use utils::*;

mod transfer_out {
    use mpl_trifle::{
        instruction::{transfer_in, transfer_out},
        state::{
            escrow_constraints::EscrowConstraintModel, transfer_effects::TransferEffects,
            trifle::Trifle,
        },
    };
    use solana_program::{borsh::try_from_slice_unchecked, program_pack::Pack};

    use super::*;

    #[tokio::test]
    async fn transfer_out_twice() {
        let mut context = program_test().start_with_context().await;

        let payer_pubkey = context.payer.pubkey().to_owned();
        let (metadata, master_edition, test_collection) =
            create_nft(&mut context, true, Some(payer_pubkey)).await;
        let test_collection = test_collection.expect("test collection should exist");
        let escrow_constraint_model_addr = create_escrow_constraint_model(
            &mut context,
            TransferEffects::new().with_track(true),
            test_collection,
            vec![metadata.mint.pubkey()],
        )
        .await;

        let (trifle_addr, escrow_addr) = create_trifle(
            &mut context,
            &metadata,
            &master_edition,
            escrow_constraint_model_addr,
            None,
        )
        .await;

        let trifle_account = context
            .banks_client
            .get_account(trifle_addr)
            .await
            .expect("trifle account should exist")
            .expect("trifle account should exist");

        let trifle_account_data: Trifle =
            try_from_slice_unchecked(&trifle_account.data).expect("should deserialize");
        println!("trifle_account: {:#?}", trifle_account_data);
        let constraint_account = context
            .banks_client
            .get_account(escrow_constraint_model_addr)
            .await
            .expect("constraint account should exist")
            .expect("constraint account should exist");
        let constraint_account_data: EscrowConstraintModel =
            try_from_slice_unchecked(&constraint_account.data).expect("should deserialize");
        println!("constraint_account: {:#?}", constraint_account_data);

        // Build the attribute
        let (attribute_metadata, _) = create_sft(&mut context, false, None).await;
        let sft_account_data = get_account(&mut context, &attribute_metadata.token.pubkey()).await;
        let sft_account: spl_token::state::Account =
            spl_token::state::Account::unpack(&sft_account_data.data).unwrap();
        println!("sft_account: {:#?}", sft_account);

        let trifle_attribute_token_account =
            spl_associated_token_account::get_associated_token_address(
                &escrow_addr,
                &attribute_metadata.mint.pubkey(),
            );

        // Do it!
        let transfer_in_ix = transfer_in(
            mpl_trifle::id(),
            trifle_addr,
            context.payer.pubkey(),
            context.payer.pubkey(),
            escrow_constraint_model_addr,
            escrow_addr,
            Some(metadata.mint.pubkey()),
            Some(metadata.token.pubkey()),
            Some(context.payer.pubkey()),
            attribute_metadata.mint.pubkey(),
            attribute_metadata.token.pubkey(),
            Some(trifle_attribute_token_account),
            Some(attribute_metadata.pubkey),
            None,
            None,
            "test".to_string(),
            2,
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
            .expect("transfer in should succeed");

        let attr_account_data = get_account(&mut context, &trifle_attribute_token_account).await;
        let attr_account: spl_token::state::Account =
            spl_token::state::Account::unpack(&attr_account_data.data).unwrap();
        println!("attr_account: {:#?}", attr_account);

        let trifle_account = context
            .banks_client
            .get_account(trifle_addr)
            .await
            .expect("trifle account should exist")
            .expect("trifle account should exist");

        let trifle_account_data: Trifle =
            try_from_slice_unchecked(&trifle_account.data).expect("should deserialize");
        println!("trifle_account: {:#?}", trifle_account_data);

        let payer_attribute_token_account =
            spl_associated_token_account::get_associated_token_address(
                &context.payer.pubkey(),
                &attribute_metadata.mint.pubkey(),
            );

        let transfer_out_ix0 = transfer_out(
            mpl_trifle::id(),
            trifle_addr,
            escrow_constraint_model_addr,
            escrow_addr,
            metadata.token.pubkey(),
            metadata.mint.pubkey(),
            metadata.pubkey,
            None,
            context.payer.pubkey(),
            context.payer.pubkey(),
            attribute_metadata.mint.pubkey(),
            trifle_attribute_token_account,
            payer_attribute_token_account,
            attribute_metadata.pubkey,
            "test".to_string(),
            1,
        );

        let transfer_out_ix1 = transfer_out(
            mpl_trifle::id(),
            trifle_addr,
            escrow_constraint_model_addr,
            escrow_addr,
            metadata.token.pubkey(),
            metadata.mint.pubkey(),
            metadata.pubkey,
            None,
            context.payer.pubkey(),
            context.payer.pubkey(),
            attribute_metadata.mint.pubkey(),
            trifle_attribute_token_account,
            payer_attribute_token_account,
            attribute_metadata.pubkey,
            "test".to_string(),
            1,
        );

        let transfer_out_tx = Transaction::new_signed_with_payer(
            &[transfer_out_ix0, transfer_out_ix1],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(transfer_out_tx)
            .await
            .expect("transfer out should succeed");

        let trifle_account = context
            .banks_client
            .get_account(trifle_addr)
            .await
            .expect("trifle account should exist")
            .expect("trifle account should exist");

        let trifle_account_data: Trifle =
            try_from_slice_unchecked(&trifle_account.data).expect("should deserialize");
        println!("trifle_account: {:#?}", trifle_account_data);
    }
}
