#![cfg(feature = "test-bpf")]
pub mod common;
pub mod utils;

use common::*;
use mpl_auctioneer::sell::config::ListingConfig;
use std::{assert_eq, time::SystemTime};
use utils::setup_functions::*;

#[tokio::test]
async fn buy_success() {
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

    let ((sell_acc, listing_config_address), sell_tx) = sell(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60) as i64,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60) as i64,
        None,
        None,
        None,
        None,
        None,
    );
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();

    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), 10000000000)
        .await
        .unwrap();
    let (_, deposit_tx) = deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer,
        1000000000,
    );
    context
        .banks_client
        .process_transaction(deposit_tx)
        .await
        .unwrap();

    let (_acc, buy_tx) = buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer,
        &sell_acc.wallet,
        &listing_config_address,
        1000000000,
    );
    context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap();
}

#[tokio::test]
async fn multiple_bids() {
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

    let ((sell_acc, listing_config_address), sell_tx) = sell(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60) as i64,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60) as i64,
        None,
        None,
        None,
        None,
        None,
    );
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();

    let buyer0 = Keypair::new();
    airdrop(&mut context, &buyer0.pubkey(), 10000000000)
        .await
        .unwrap();
    let (_, deposit_tx0) = deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer0,
        1000000000,
    );
    context
        .banks_client
        .process_transaction(deposit_tx0)
        .await
        .unwrap();

    let (_acc0, buy_tx0) = buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer0,
        &sell_acc.wallet,
        &listing_config_address,
        1000000000,
    );
    context
        .banks_client
        .process_transaction(buy_tx0)
        .await
        .unwrap();

    let buyer1 = Keypair::new();
    airdrop(&mut context, &buyer1.pubkey(), 100000000000)
        .await
        .unwrap();
    let (_, deposit_tx1) = deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer1,
        10000000000,
    );
    context
        .banks_client
        .process_transaction(deposit_tx1)
        .await
        .unwrap();

    let (_acc1, buy_tx1) = buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer1,
        &sell_acc.wallet,
        &listing_config_address,
        10000000000,
    );
    context
        .banks_client
        .process_transaction(buy_tx1)
        .await
        .unwrap();

    let listing = context
        .banks_client
        .get_account(listing_config_address)
        .await
        .unwrap()
        .unwrap()
        .data;
    let config = ListingConfig::try_deserialize(&mut listing.as_ref()).unwrap();
    assert_eq!(config.highest_bid.amount, 10000000000);
}

#[tokio::test]
async fn buy_below_reserve_failure() {
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

    let ((sell_acc, listing_config_address), sell_tx) = sell(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60) as i64,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60) as i64,
        Some(1000000001),
        None,
        None,
        None,
        None,
    );
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();

    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), 10000000000)
        .await
        .unwrap();
    let (_, deposit_tx) = deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer,
        1000000000,
    );
    context
        .banks_client
        .process_transaction(deposit_tx)
        .await
        .unwrap();

    let (_acc, buy_tx) = buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer,
        &sell_acc.wallet,
        &listing_config_address,
        1000000000,
    );
    let result = context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap_err();
    assert_error!(result, BELOW_RESERVE_PRICE);
}

#[tokio::test]
async fn buy_above_reserve_success() {
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

    let ((sell_acc, listing_config_address), sell_tx) = sell(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60) as i64,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60) as i64,
        Some(1000000000),
        None,
        None,
        None,
        None,
    );
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();

    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), 10000000000)
        .await
        .unwrap();
    let (_, deposit_tx) = deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer,
        1000000000,
    );
    context
        .banks_client
        .process_transaction(deposit_tx)
        .await
        .unwrap();

    let (_acc, buy_tx) = buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer,
        &sell_acc.wallet,
        &listing_config_address,
        1000000000,
    );
    context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap();
}

#[tokio::test]
async fn multiple_bids_increment_failure() {
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

    let ((sell_acc, listing_config_address), sell_tx) = sell(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60) as i64,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60) as i64,
        None,
        Some(2000000000),
        None,
        None,
        None,
    );
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();

    let buyer0 = Keypair::new();
    airdrop(&mut context, &buyer0.pubkey(), 10000000000)
        .await
        .unwrap();
    let (_, deposit_tx0) = deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer0,
        1000000000,
    );
    context
        .banks_client
        .process_transaction(deposit_tx0)
        .await
        .unwrap();

    let (_acc0, buy_tx0) = buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer0,
        &sell_acc.wallet,
        &listing_config_address,
        1000000000,
    );
    context
        .banks_client
        .process_transaction(buy_tx0)
        .await
        .unwrap();

    let buyer1 = Keypair::new();
    airdrop(&mut context, &buyer1.pubkey(), 100000000000)
        .await
        .unwrap();
    let (_, deposit_tx1) = deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer1,
        1000000001,
    );
    context
        .banks_client
        .process_transaction(deposit_tx1)
        .await
        .unwrap();

    let (_acc1, buy_tx1) = buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer1,
        &sell_acc.wallet,
        &listing_config_address,
        1000000001,
    );
    let result = context
        .banks_client
        .process_transaction(buy_tx1)
        .await
        .unwrap_err();
    assert_error!(result, BELOW_BID_INCREMENT);

    let listing = context
        .banks_client
        .get_account(listing_config_address)
        .await
        .unwrap()
        .unwrap()
        .data;
    let config = ListingConfig::try_deserialize(&mut listing.as_ref()).unwrap();
    assert_eq!(config.highest_bid.amount, 1000000000);
}

#[tokio::test]
async fn multiple_bids_increment_success() {
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

    let ((sell_acc, listing_config_address), sell_tx) = sell(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60) as i64,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60) as i64,
        None,
        Some(2000000000),
        None,
        None,
        None,
    );
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();

    let buyer0 = Keypair::new();
    airdrop(&mut context, &buyer0.pubkey(), 10000000000)
        .await
        .unwrap();
    let (_, deposit_tx0) = deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer0,
        1000000000,
    );
    context
        .banks_client
        .process_transaction(deposit_tx0)
        .await
        .unwrap();

    let (_acc0, buy_tx0) = buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer0,
        &sell_acc.wallet,
        &listing_config_address,
        1000000000,
    );
    context
        .banks_client
        .process_transaction(buy_tx0)
        .await
        .unwrap();

    let buyer1 = Keypair::new();
    airdrop(&mut context, &buyer1.pubkey(), 100000000000)
        .await
        .unwrap();
    let (_, deposit_tx1) = deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer1,
        30000000000,
    );
    context
        .banks_client
        .process_transaction(deposit_tx1)
        .await
        .unwrap();

    let (_acc1, buy_tx1) = buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer1,
        &sell_acc.wallet,
        &listing_config_address,
        30000000000,
    );
    context
        .banks_client
        .process_transaction(buy_tx1)
        .await
        .unwrap();

    let listing = context
        .banks_client
        .get_account(listing_config_address)
        .await
        .unwrap()
        .unwrap()
        .data;
    let config = ListingConfig::try_deserialize(&mut listing.as_ref()).unwrap();
    assert_eq!(config.highest_bid.amount, 30000000000);
}

#[tokio::test]
async fn multiple_bids_time_ext_success() {
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

    let ((sell_acc, listing_config_address), sell_tx) = sell(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60) as i64,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60) as i64,
        None,
        None,
        Some(60),
        Some(60),
        None,
    );
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();

    context.warp_to_slot(1 * 400).unwrap();

    let listing0 = context
        .banks_client
        .get_account(listing_config_address)
        .await
        .unwrap()
        .unwrap()
        .data;

    let config0 = ListingConfig::try_deserialize(&mut listing0.as_ref()).unwrap();
    let end_time_t0 = config0.end_time;

    let buyer0 = Keypair::new();
    airdrop(&mut context, &buyer0.pubkey(), 10000000000)
        .await
        .unwrap();
    let (_, deposit_tx0) = deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer0,
        1000000000,
    );
    context
        .banks_client
        .process_transaction(deposit_tx0)
        .await
        .unwrap();

    let (_acc0, buy_tx0) = buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer0,
        &sell_acc.wallet,
        &listing_config_address,
        1000000000,
    );
    context
        .banks_client
        .process_transaction(buy_tx0)
        .await
        .unwrap();

    let listing1 = context
        .banks_client
        .get_account(listing_config_address)
        .await
        .unwrap()
        .unwrap()
        .data;

    let config1 = ListingConfig::try_deserialize(&mut listing1.as_ref()).unwrap();
    assert_eq!(config1.end_time, end_time_t0 + 60);

    context.warp_to_slot(121 * 400).unwrap();

    let buyer1 = Keypair::new();
    airdrop(&mut context, &buyer1.pubkey(), 100000000000)
        .await
        .unwrap();
    let (_, deposit_tx1) = deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer1,
        10000000000,
    );
    context
        .banks_client
        .process_transaction(deposit_tx1)
        .await
        .unwrap();

    let (_acc1, buy_tx1) = buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer1,
        &sell_acc.wallet,
        &listing_config_address,
        10000000000,
    );
    context
        .banks_client
        .process_transaction(buy_tx1)
        .await
        .unwrap();

    let listing2 = context
        .banks_client
        .get_account(listing_config_address)
        .await
        .unwrap()
        .unwrap()
        .data;

    let config2 = ListingConfig::try_deserialize(&mut listing2.as_ref()).unwrap();
    assert_eq!(config2.end_time, end_time_t0 + 120);
}
