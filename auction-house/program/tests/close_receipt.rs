#![cfg(feature = "test-bpf")]
mod utils;
use anchor_lang::AccountDeserialize;
use mpl_auction_house::{pda::find_trade_state_address, Purchase, Receipt};
use mpl_testing_utils::{assert_error, solana::airdrop, utils::Metadata};
use solana_program::instruction::InstructionError;
use solana_program_test::*;
use solana_sdk::{
    signature::Keypair, signer::Signer, transaction::TransactionError, transport::TransportError,
};
use spl_associated_token_account::get_associated_token_address;
use std::assert_eq;
use utils::setup_functions::*;

#[tokio::test]
async fn success_close_receipt() {
    let mut context = auction_house_program_test().start_with_context().await;
    let price = 100_000_000;

    let (ah, ahkey, authority) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();

    airdrop(&mut context, &test_metadata.token.pubkey(), 10_000_000_000)
        .await
        .unwrap();

    airdrop(&mut context, &ah.auction_house_fee_account, 10_000_000_000)
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

    let (sell_acc, sell_tx) = sell(&mut context, &ahkey, &ah, &test_metadata, price);
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();

    let token =
        get_associated_token_address(&test_metadata.token.pubkey(), &test_metadata.mint.pubkey());
    let (trade_state, ts_bump) = find_trade_state_address(
        &test_metadata.token.pubkey(),
        &ahkey,
        &token,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        100_000_000,
        1,
    );

    let (_receipt_acc, print_receipt_tx) = print_receipt(
        &mut context,
        &ahkey,
        &token,
        &trade_state,
        ts_bump,
        &test_metadata.token,
        100_000_000,
    );

    context
        .banks_client
        .process_transaction(print_receipt_tx)
        .await
        .unwrap();

    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), 10_000_000_000)
        .await
        .unwrap();
    let (buy_acc, buy_tx) = buy(&mut context, &ahkey, &ah, &test_metadata, &buyer, price);

    context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap();

    let (execute_sale_acc, execute_sale_tx) = execute_sale_with_receipt(
        &mut context,
        &ahkey,
        &ah,
        &authority,
        &test_metadata,
        &buyer.pubkey(),
        &test_metadata.token.pubkey(),
        &sell_acc.token_account,
        &sell_acc.seller_trade_state,
        &buy_acc.buyer_trade_state,
        1,
        price,
    );

    context
        .banks_client
        .process_transaction(execute_sale_tx)
        .await
        .unwrap();

    let (receipt_acc, receipt_tx) = burn_receipt(&mut context, &ahkey, &ah, &test_metadata, price);

    context
        .banks_client
        .process_transaction(receipt_tx)
        .await
        .unwrap();

    let receipt_closed_account = context
        .banks_client
        .get_account(receipt_acc.receipt)
        .await
        .expect("error getting receipt")
        .expect("no data for receipt");

    let receipt_closed =
        Receipt::try_deserialize(&mut receipt_closed_account.data.as_ref()).unwrap();

    let purchase_receipt_account = context
        .banks_client
        .get_account(execute_sale_acc.purchase_receipt)
        .await
        .expect("error getting purchase receipt")
        .expect("no purchase receipt data");

    let purchase_receipt =
        Purchase::try_deserialize(&mut purchase_receipt_account.data.as_ref()).unwrap();

    assert_eq!(receipt_closed.closed, true);
    assert_eq!(purchase_receipt.price, price);
    assert_eq!(purchase_receipt.buyer, buyer.pubkey());
    assert_eq!(purchase_receipt.seller, test_metadata.token.pubkey());
    assert_eq!(purchase_receipt.auction_house, ahkey);
    ()
}

#[tokio::test]
async fn fail_no_closing_active_trade_state_receipts() {
    let mut context = auction_house_program_test().start_with_context().await;
    let price = 100_000_000;
    let (ah, ahkey, _authority) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();

    airdrop(&mut context, &test_metadata.token.pubkey(), 10_000_000_000)
        .await
        .unwrap();

    airdrop(&mut context, &ah.auction_house_fee_account, 10_000_000_000)
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

    let (sell_acc, sell_tx) = sell(&mut context, &ahkey, &ah, &test_metadata, price);
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();

    let token =
        get_associated_token_address(&test_metadata.token.pubkey(), &test_metadata.mint.pubkey());
    let (_trade_state, ts_bump) = find_trade_state_address(
        &sell_acc.wallet,
        &ahkey,
        &token,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        price,
        1,
    );

    let (_print_receipt_acc, print_receipt_tx) = print_receipt(
        &mut context,
        &ahkey,
        &token,
        &sell_acc.seller_trade_state,
        ts_bump,
        &test_metadata.token,
        price,
    );

    context
        .banks_client
        .process_transaction(print_receipt_tx)
        .await
        .unwrap();

    let (_receipt_acc, receipt_tx) = burn_receipt(&mut context, &ahkey, &ah, &test_metadata, price);

    let burn_transaction_err = context
        .banks_client
        .process_transaction(receipt_tx)
        .await
        .unwrap_err();

    assert_error!(burn_transaction_err, 6025);

    ()
}
