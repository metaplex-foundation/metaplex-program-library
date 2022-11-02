#![cfg(feature = "test-bpf")]
pub mod common;
pub mod utils;

use anchor_lang::AccountDeserialize;
use common::*;
use utils::setup_functions::*;

use mpl_testing_utils::{solana::airdrop, utils::Metadata};
use solana_sdk::signer::Signer;
use std::assert_eq;

#[tokio::test]
async fn success() {
    // Have an existing public buy order open then:
    // Instruction order:
    // * sell, print_listing_receipt, execute_sale, print_purchase_receipt
    let mut context = auction_house_program_test().start_with_context().await;

    let price = 500_732_504;

    let (ah, ahkey, authority) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();

    let test_metadata = Metadata::new();
    airdrop(&mut context, &test_metadata.token.pubkey(), ONE_SOL)
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
    airdrop(&mut context, &buyer.pubkey(), ONE_SOL * 10)
        .await
        .unwrap();
    let (_, deposit_tx) = deposit(&mut context, &ahkey, &ah, &test_metadata, &buyer, price);
    context
        .banks_client
        .process_transaction(deposit_tx)
        .await
        .unwrap();

    let ((buy_accounts, print_bid_acc), buy_tx) = public_buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer,
        price,
    );
    context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap();
    let bts = context
        .banks_client
        .get_account(buy_accounts.buyer_trade_state)
        .await
        .expect("Error Getting Trade State")
        .expect("Trade State Empty");
    assert_eq!(bts.data.len(), 1);

    let bid_receipt_account = context
        .banks_client
        .get_account(print_bid_acc.receipt)
        .await
        .expect("Error Getting Public Bid Receipt")
        .expect("Public Bid Empty");

    let bid_receipt = BidReceipt::try_deserialize(&mut bid_receipt_account.data.as_ref()).unwrap();

    assert_eq!(bid_receipt.price, price);
    assert_eq!(bid_receipt.auction_house, buy_accounts.auction_house);
    assert_eq!(bid_receipt.metadata, buy_accounts.metadata);
    assert_eq!(bid_receipt.token_account, None); // Public bid so no token account.
    assert_eq!(bid_receipt.buyer, buy_accounts.wallet);
    assert_eq!(bid_receipt.trade_state, buy_accounts.buyer_trade_state);
    assert_eq!(bid_receipt.token_size, 1);
    assert_eq!(bid_receipt.purchase_receipt, None);
    assert_eq!(bid_receipt.bookkeeper, buyer.pubkey());

    airdrop(&mut context, &ah.auction_house_fee_account, 10_000_000_000)
        .await
        .unwrap();

    // Build instructions
    let (sell_ix, sell_accounts) = sell_ix(&ahkey, &ah, &test_metadata, price, 1);
    let print_listing_receipt_ix = print_listing_receipt_ix(&ahkey, &ah, &test_metadata, price, 1);

    let execute_sale_ix = execute_sale_ix(
        &ahkey,
        &ah,
        &test_metadata,
        &buyer.pubkey(),
        &sell_accounts.wallet,
        &sell_accounts.token_account,
        &sell_accounts.seller_trade_state,
        &buy_accounts.buyer_trade_state,
        1,
        price,
    );
    let print_purchase_receipt_ix = print_purchase_receipt_ix(
        &ah.authority,
        &sell_accounts.seller_trade_state,
        &buy_accounts.buyer_trade_state,
    );

    // Compose transaction
    let tx = Transaction::new_signed_with_payer(
        &[
            sell_ix,
            print_listing_receipt_ix,
            execute_sale_ix,
            print_purchase_receipt_ix,
        ],
        Some(&test_metadata.token.pubkey()),
        &[&test_metadata.token, &authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();
}
