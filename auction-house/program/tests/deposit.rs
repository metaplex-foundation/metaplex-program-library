#![cfg(feature = "test-bpf")]
pub mod utils;
use mpl_auction_house::pda::find_trade_state_address;
use mpl_testing_utils::{solana::airdrop, utils::Metadata};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};
use spl_associated_token_account::get_associated_token_address;
use std::assert_eq;
use utils::setup_functions::*;

#[tokio::test]
async fn deposit_success() {
    let mut context = auction_house_program_test().start_with_context().await;
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
        )
        .await
        .unwrap();
    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), 3_000_000_000)
        .await
        .unwrap();
    for dedicated_escrow in [false, true] {
        let sale_price = 1_000_000_000;
        let seller_token_account = get_associated_token_address(
            &test_metadata.token.pubkey(),
            &test_metadata.mint.pubkey(),
        );
        let (trade_state, _) = find_trade_state_address(
            &buyer.pubkey(),
            &ahkey,
            &seller_token_account,
            &ah.treasury_mint,
            &test_metadata.mint.pubkey(),
            sale_price,
            1,
        );

        let buyer_trade_state = if dedicated_escrow {
            Some(trade_state)
        } else {
            None
        };

        let (acc, deposit_tx) = deposit(
            &mut context,
            &ahkey,
            &ah,
            &test_metadata,
            &buyer,
            sale_price,
            buyer_trade_state,
        );

        context
            .banks_client
            .process_transaction(deposit_tx)
            .await
            .unwrap();
        let escrow = context
            .banks_client
            .get_account(acc.escrow_payment_account)
            .await
            .expect("Error Getting Escrow")
            .expect("Trade State Escrow");
        assert_eq!(escrow.lamports, sale_price + 890880);
    }
}
