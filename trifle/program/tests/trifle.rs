#![cfg(feature = "test-bpf")]

pub mod utils;

use solana_program::borsh::try_from_slice_unchecked;
use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::Transaction};
use utils::*;

mod trifle {
    use borsh::{BorshDeserialize, BorshSerialize};
    use mpl_token_metadata::state::EscrowAuthority;
    use mpl_trifle::{
        instruction::{
            add_constraint_to_escrow_constraint_model, create_escrow_constraint_model_account,
            create_trifle_account,
        },
        pda::{find_escrow_constraint_model_address, find_trifle_address},
        state::escrow_constraints::{EscrowConstraint, EscrowConstraintType},
    };

    use super::*;

    #[tokio::test]
    async fn test_create_trifle_account() {
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

        let (escrow_constraint_model_addr, _) =
            find_escrow_constraint_model_address(&context.payer.pubkey(), "Test");

        let create_constraint_model_ix = create_escrow_constraint_model_account(
            &mpl_trifle::id(),
            &escrow_constraint_model_addr,
            &context.payer.pubkey(),
            &context.payer.pubkey(),
            "Test".to_string(),
        );

        let constraint = EscrowConstraint {
            name: "test".to_string(),
            token_limit: 1,
            constraint_type: EscrowConstraintType::None,
        };

        let add_constraint_ix = add_constraint_to_escrow_constraint_model(
            &mpl_trifle::id(),
            &escrow_constraint_model_addr,
            &context.payer.pubkey(),
            &context.payer.pubkey(),
            constraint,
        );

        let tx = Transaction::new_signed_with_payer(
            &[create_constraint_model_ix, add_constraint_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        println!("before------------------------------------------------------------------");
        let (trifle_addr, _) = find_trifle_address(
            &metadata.mint.pubkey(),
            &context.payer.pubkey(),
            &escrow_constraint_model_addr,
        );
        println!("after------------------------------------------------------------------");

        println!("before------------------------------------------------------------------");
        let (escrow_addr, _) = mpl_token_metadata::escrow::pda::find_escrow_account(
            &context.payer.pubkey(),
            &EscrowAuthority::Creator(trifle_addr.to_owned()),
        );
        println!("after------------------------------------------------------------------");

        let token_account_addr =
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &escrow_addr,
                &metadata.mint.pubkey(),
                &spl_token::id(),
            );

        let create_trifle_account_ix = create_trifle_account(
            &mpl_trifle::id(),
            &escrow_addr,
            &metadata.pubkey,
            &metadata.mint.pubkey(),
            &token_account_addr,
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
    }
}
