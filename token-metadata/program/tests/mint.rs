#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::error::MetadataError;
use mpl_token_metadata::{instruction};
use num_traits::FromPrimitive;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};
use utils::*;
use mpl_token_metadata::state::MasterEditionSupply;

mod mint {

    use super::*;

    #[tokio::test]
    async fn mint_success() {
        let mut context = program_test().start_with_context().await;
        let payer = context.payer.pubkey();

        airdrop(&mut context, &payer, 10000000)
            .await
            .unwrap();

        let mint = Keypair::new();

        let ix = instruction::mint(
            mint.pubkey(),
            payer,
            payer,
            payer,
            "Test".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            None,
            10,
            true,
            true,
            None,
            None,
            MasterEditionSupply::Single
        );

        let mint_tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer),
            &[&context.payer, &mint],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(mint_tx)
            .await
            .unwrap();

    }
}
