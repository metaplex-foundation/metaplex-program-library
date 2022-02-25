#![cfg(feature = "test-bpf")]
mod utils;
use anchor_lang::AccountDeserialize;

use claim::assert_none;
use mpl_auction_house::{pda::find_public_bid_trade_state_address, receipt::PublicBid};
use mpl_testing_utils::{assert_error, solana::airdrop, utils::Metadata};
use solana_program::instruction::InstructionError;
use solana_program_test::*;
use solana_sdk::{
    signature::Keypair, signer::Signer, sysvar::clock::Clock, transaction::TransactionError,
    transport::TransportError,
};
use spl_associated_token_account::get_associated_token_address;

use std::assert_eq;
use utils::setup_functions::*;

#[tokio::test]
async fn print_public_bid_receipt_success() {
    let mut context = auction_house_program_test().start_with_context().await;

    let (ah, ahkey, _authority) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();
    let buyer = Keypair::new();
    let buyer_key = buyer.pubkey();

    airdrop(&mut context, &buyer.pubkey(), 10_000_000_000)
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

    let owner = &test_metadata.token;
    let owner_key = owner.pubkey();

    let (buy_acc, buy_tx) = public_buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &owner_key,
        &buyer,
        100_000_000,
    );
    context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap();

    let owner_token_account =
        get_associated_token_address(&owner_key, &test_metadata.mint.pubkey());
    let (trade_state, ts_bump) = find_public_bid_trade_state_address(
        &buyer_key,
        &ahkey,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        100_000_000,
        1,
    );

    let (receipt_acc, receipt_tx) = print_public_bid_receipt(
        &mut context,
        &buyer,
        &ahkey,
        &owner_token_account,
        &trade_state,
        ts_bump,
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

    let bid = PublicBid::try_deserialize(&mut receipt_account.data.as_ref()).unwrap();
    let timestamp = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp;

    assert_eq!(bid.bookkeeper, buyer_key);
    assert_eq!(bid.token_mint, test_metadata.mint.pubkey());
    assert_eq!(bid.trade_state, buy_acc.buyer_trade_state);
    assert_eq!(bid.auction_house, buy_acc.auction_house);
    assert_eq!(bid.wallet, buy_acc.wallet);
    assert_eq!(bid.activated_at, Some(timestamp));
    assert_none!(bid.closed_at);
    assert_eq!(bid.price, 100_000_000);
    assert_eq!(bid.token_size, 1);
    ()
}

#[tokio::test]
async fn failure_print_public_bid_receipt_trade_state_mismatch() {
    let mut context = auction_house_program_test().start_with_context().await;
    let (ah, ahkey, _authority) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();
    let buyer = Keypair::new();
    let buyer_key = buyer.pubkey();

    airdrop(&mut context, &buyer_key, 10_000_000_000)
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

    let owner = &test_metadata.token;
    let owner_key = owner.pubkey();

    let (_buy_acc, buy_tx) = public_buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &owner_key,
        &buyer,
        100_000_000,
    );
    context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap();

    let token = get_associated_token_address(&owner_key, &test_metadata.mint.pubkey());
    let (trade_state, ts_bump) = find_public_bid_trade_state_address(
        &buyer_key,
        &ahkey,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        100_000_000,
        1,
    );

    let (_receipt_acc, receipt_tx) = print_public_bid_receipt(
        &mut context,
        &buyer,
        &ahkey,
        &token,
        &trade_state,
        ts_bump,
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
