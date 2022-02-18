#![cfg(feature = "test-bpf")]
mod utils;
use anchor_lang::AccountDeserialize;
use claim::assert_some;
use mpl_auction_house::{
    pda::{find_public_bid_trade_state_address, find_trade_state_address},
    Listing,
};
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
async fn success_close_listing_receipt_after_sale() {
    let mut context = auction_house_program_test().start_with_context().await;
    let price = 100_000_000;
    let buyer = Keypair::new();
    let buyer_key = buyer.pubkey();

    let (ah, ahkey, authority) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();

    airdrop(&mut context, &buyer_key, 10_000_000_000)
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

    let owner = &test_metadata.token;
    let owner_key = owner.pubkey();

    airdrop(&mut context, &test_metadata.token.pubkey(), 10_000_000_000)
        .await
        .unwrap();

    let (sell_acc, sell_tx) = sell(&mut context, &ahkey, &ah, &test_metadata, price);
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();

    let token = get_associated_token_address(&owner_key, &test_metadata.mint.pubkey());

    let (seller_trade_state, seller_trade_state_bump) = find_trade_state_address(
        &owner_key,
        &ahkey,
        &token,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        price,
        1,
    );
    let (_receipt_acc, print_receipt_tx) = print_listing_receipt(
        &mut context,
        &owner,
        &ahkey,
        &token,
        &seller_trade_state,
        seller_trade_state_bump,
        price,
    );

    context
        .banks_client
        .process_transaction(print_receipt_tx)
        .await
        .unwrap();

    let (trade_state, ts_bump) = find_public_bid_trade_state_address(
        &buyer_key,
        &ahkey,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        price,
        1,
    );

    let (buy_acc, buy_tx) = public_buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &owner_key,
        &buyer,
        price,
    );

    context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap();

    let (_execute_sall_acc, execute_sale_tx) = execute_sale(
        &mut context,
        &ahkey,
        &ah,
        &authority,
        &test_metadata,
        &buyer_key,
        &owner_key,
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

    let (receipt_acc, receipt_tx) = close_listing_receipt(
        &mut context,
        &test_metadata.token,
        &token,
        &ahkey,
        &ah,
        &test_metadata,
        price,
    );

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

    let listing = Listing::try_deserialize(&mut receipt_closed_account.data.as_ref()).unwrap();
    let timestamp = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp;

    assert_eq!(listing.closed_at, Some(timestamp));
    assert_some!(listing.activated_at);

    ()
}

#[tokio::test]
async fn fail_no_closing_listing_receipts_when_trade_state_is_available() {
    let mut context = auction_house_program_test().start_with_context().await;
    let price = 100_000_000;
    let buyer = Keypair::new();
    let buyer_key = buyer.pubkey();

    let (ah, ahkey, authority) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();

    airdrop(&mut context, &buyer_key, 10_000_000_000)
        .await
        .unwrap();
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

    let owner = &test_metadata.token;
    let owner_key = owner.pubkey();

    let (sell_acc, sell_tx) = sell(&mut context, &ahkey, &ah, &test_metadata, price);
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();

    let token = get_associated_token_address(&owner_key, &test_metadata.mint.pubkey());

    let (seller_trade_state, seller_trade_state_bump) = find_trade_state_address(
        &owner_key,
        &ahkey,
        &token,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        price,
        1,
    );
    let (_receipt_acc, print_receipt_tx) = print_listing_receipt(
        &mut context,
        &owner,
        &ahkey,
        &token,
        &seller_trade_state,
        seller_trade_state_bump,
        price,
    );

    context
        .banks_client
        .process_transaction(print_receipt_tx)
        .await
        .unwrap();

    let (receipt_acc, receipt_tx) = close_listing_receipt(
        &mut context,
        &test_metadata.token,
        &token,
        &ahkey,
        &ah,
        &test_metadata,
        price,
    );

    let close_transaction_err = context
        .banks_client
        .process_transaction(receipt_tx)
        .await
        .unwrap_err();

    assert_error!(close_transaction_err, 6025);

    ()
}
