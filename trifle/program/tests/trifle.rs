#![cfg(feature = "test-bpf")]

pub mod utils;

use solana_program::borsh::try_from_slice_unchecked;
use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::Transaction};
use utils::*;

mod trifle {
    use borsh::{BorshDeserialize, BorshSerialize};
    use mpl_trifle::{
        instruction::{create_escrow_constraint_model_account, TrifleInstruction},
        pda::find_escrow_constraint_model_address,
    };

    use super::*;

    #[tokio::test]
    async fn create_trifle_account() {
        let mut context = program_test().start_with_context().await;

        let metadata = Metadata::new();
        let master_edition = MasterEditionV2::new(&metadata);
        metadata
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

        master_edition
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let (escrow_constraint_model_addr, _) = find_escrow_constraint_model_address(
            &context.payer.pubkey(),
            "Test",
            &mpl_trifle::id(),
        );

        let create_constraint_model_ix = create_escrow_constraint_model_account(
            &mpl_trifle::id(),
            &escrow_constraint_model_addr,
            &context.payer.pubkey(),
            &context.payer.pubkey(),
            "Test".to_string(),
        );

        let tx = Transaction::new_signed_with_payer(
            &[create_constraint_model_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();
    }
}
