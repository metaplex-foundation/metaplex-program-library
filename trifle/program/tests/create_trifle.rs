#![cfg(feature = "test-bpf")]

pub mod utils;

use solana_program_test::*;
use solana_sdk::signer::Signer;
use utils::*;

mod create_trifle {
    use mpl_trifle::state::{
        escrow_constraints::EscrowConstraintModel, transfer_effects::TransferEffects,
        trifle::Trifle,
    };
    use solana_program::borsh::try_from_slice_unchecked;

    use super::*;

    #[tokio::test]
    async fn create_trifle_pass() {
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

        let (trifle_addr, _escrow_addr) = create_trifle(
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
    }
}
