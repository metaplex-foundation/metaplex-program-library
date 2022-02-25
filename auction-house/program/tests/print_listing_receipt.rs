#![cfg(feature = "test-bpf")]
mod utils;
use anchor_lang::AccountDeserialize;
use claim::assert_none;
use mpl_auction_house::{pda::find_trade_state_address, receipt::Listing};
use mpl_testing_utils::{assert_error, solana::airdrop, utils::Metadata};
use solana_program::instruction::InstructionError;
use solana_program_test::*;
use solana_sdk::{
    signer::Signer, sysvar::clock::Clock, transaction::TransactionError,
    transport::TransportError,
};
use spl_associated_token_account::get_associated_token_address;
use std::assert_eq;
use utils::setup_functions::*;

#[tokio::test]
async fn print_listing_receipt_success() {
    let mut context = auction_house_program_test().start_with_context().await;

    let (ah, ahkey, _authority) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let price = 100_000_000;
    let test_metadata = Metadata::new();

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

    airdrop(&mut context, &owner_key, 10_000_000_000)
        .await
        .unwrap();

    let (sell_acc, sell_tx) = sell(&mut context, &ahkey, &ah, &test_metadata, price);
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();

    let owner_token_account =
        get_associated_token_address(&owner_key, &test_metadata.mint.pubkey());
    let (trade_state, ts_bump) = find_trade_state_address(
        &owner_key,
        &ahkey,
        &owner_token_account,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        price,
        1,
    );

    let (receipt_acc, receipt_tx) = print_listing_receipt(
        &mut context,
        &owner,
        &ahkey,
        &owner_token_account,
        &trade_state,
        ts_bump,
        price,
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

    let listing = Listing::try_deserialize(&mut receipt_account.data.as_ref()).unwrap();
    let timestamp = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp;

    assert_eq!(listing.bookkeeper, owner_key);
    assert_eq!(listing.token_mint, test_metadata.mint.pubkey());
    assert_eq!(listing.trade_state, sell_acc.seller_trade_state);
    assert_eq!(listing.auction_house, sell_acc.auction_house);
    assert_eq!(listing.seller, sell_acc.wallet);
    assert_eq!(listing.activated_at, Some(timestamp));
    assert_none!(listing.closed_at);
    assert_eq!(listing.price, 100_000_000);
    assert_eq!(listing.token_size, 1);
    ()
}

#[tokio::test]
async fn failure_print_listing_receipt_trade_state_mismatch() {
    let mut context = auction_house_program_test().start_with_context().await;
    let (ah, ahkey, _authority) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();

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

    let price = 100_000_000;
    let owner = &test_metadata.token;
    let owner_key = owner.pubkey();

    airdrop(&mut context, &owner_key, 10_000_000_000)
        .await
        .unwrap();

    let (_sell_acc, sell_tx) = sell(&mut context, &ahkey, &ah, &test_metadata, price);
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();

    let token = get_associated_token_address(&owner_key, &test_metadata.mint.pubkey());
    let (trade_state, ts_bump) = find_trade_state_address(
        &owner_key,
        &ahkey,
        &token,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        price,
        1,
    );

    let (_receipt_acc, receipt_tx) = print_listing_receipt(
        &mut context,
        &owner,
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
