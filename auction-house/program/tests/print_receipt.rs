#![cfg(feature = "test-bpf")]
mod utils;
use anchor_lang::AccountDeserialize;

use mpl_auction_house::{pda::find_trade_state_address, Receipt};
use mpl_testing_utils::{assert_error, solana::airdrop, utils::Metadata};
use solana_program::instruction::InstructionError;
use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::TransactionError, transport::TransportError};
use spl_associated_token_account::get_associated_token_address;

use std::assert_eq;
use utils::setup_functions::*;

#[tokio::test]
async fn print_receipt_success() {
    let mut context = auction_house_program_test().start_with_context().await;

    let (ah, ahkey, _authority) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();

    airdrop(&mut context, &test_metadata.token.pubkey(), 10_000_000_000)
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

    let (sell_acc, sell_tx) = sell(&mut context, &ahkey, &ah, &test_metadata, 100_000_000);
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

    let (receipt_acc, receipt_tx) = print_receipt(
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
        .process_transaction(receipt_tx)
        .await
        .unwrap();

    let receipt_account = context
        .banks_client
        .get_account(receipt_acc.receipt)
        .await
        .expect("error getting receipt")
        .expect("no receipt data");

    let receipt = Receipt::try_deserialize(&mut receipt_account.data.as_ref()).unwrap();

    assert_eq!(receipt.bookkeeper, test_metadata.token.pubkey());
    assert_eq!(receipt.trade_state, sell_acc.seller_trade_state);
    assert_eq!(receipt.token_account, sell_acc.token_account);
    assert_eq!(receipt.auction_house, sell_acc.auction_house);
    assert_eq!(receipt.wallet, sell_acc.wallet);
    assert_eq!(receipt.price, 100_000_000);
    assert_eq!(receipt.token_size, 1);
    ()
}

#[tokio::test]
async fn failure_print_receipt_trade_state_mismatch() {
    let mut context = auction_house_program_test().start_with_context().await;
    let (ah, ahkey, _authority) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();

    airdrop(&mut context, &test_metadata.token.pubkey(), 10_000_000_000)
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

    let (_sell_acc, sell_tx) = sell(&mut context, &ahkey, &ah, &test_metadata, 100_000_000);
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

    let (_receipt_acc, receipt_tx) = print_receipt(
        &mut context,
        &ahkey,
        &token,
        &trade_state,
        ts_bump,
        &test_metadata.token,
        1,
    );

    let print_receipt_err = context
        .banks_client
        .process_transaction(receipt_tx)
        .await
        .unwrap_err();

    assert_error!(print_receipt_err, 2006);

    ()
}
