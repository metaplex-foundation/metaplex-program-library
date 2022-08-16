#![cfg(feature = "test-bpf")]
pub mod common;
pub mod utils;

use common::*;
use mpl_testing_utils::{solana::airdrop, utils::Metadata};
use solana_sdk::{signature::Keypair, signer::Signer};
use std::assert_eq;
use utils::setup_functions::*;

#[tokio::test]
async fn deposit_success() {
    let mut context = auctioneer_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, _) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();
    airdrop(&mut context, &test_metadata.token.pubkey(), 1000000000)
        .await
        .unwrap();
    test_metadata
        .create(
            &mut context,
            "Test".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            None,
            10,
            false,
            1,
        )
        .await
        .unwrap();
    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), ONE_SOL * 2)
        .await
        .unwrap();
    let (acc, deposit_tx) = deposit(&mut context, &ahkey, &ah, &test_metadata, &buyer, ONE_SOL);

    context
        .banks_client
        .process_transaction(deposit_tx)
        .await
        .unwrap();

    let escrow_payment_account_data_len = 0;
    let rent = context.banks_client.get_rent().await.unwrap();
    let rent_exempt_min: u64 = rent.minimum_balance(escrow_payment_account_data_len);

    let escrow = context
        .banks_client
        .get_account(acc.escrow_payment_account)
        .await
        .expect("Error Getting Escrow")
        .expect("Trade State Escrow");
    assert_eq!(escrow.lamports, ONE_SOL + rent_exempt_min);
}
