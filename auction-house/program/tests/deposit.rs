#![cfg(feature = "test-bpf")]
pub mod common;
pub mod utils;

use common::*;
use mpl_auction_house::pda::find_auctioneer_pda;
use mpl_testing_utils::{solana::airdrop, utils::Metadata};
use solana_sdk::{signature::Keypair, signer::Signer};
use std::assert_eq;
use utils::{helpers::default_scopes, setup_functions::*};

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
    airdrop(&mut context, &buyer.pubkey(), 2000000000)
        .await
        .unwrap();
    let (acc, deposit_tx) = deposit(
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
    let escrow = context
        .banks_client
        .get_account(acc.escrow_payment_account)
        .await
        .expect("Error Getting Escrow")
        .expect("Trade State Escrow");
    assert_eq!(escrow.lamports, 1000000000);
}

#[tokio::test]
async fn auction_deposit_success() {
    let mut context = auction_house_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, ah_auth) = existing_auction_house_test_context(&mut context)
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

    // Delegate external auctioneer authority.
    let auctioneer_authority = Keypair::new();
    let (auctioneer_pda, auctioneer_pda_bump) =
        find_auctioneer_pda(&ahkey, &auctioneer_authority.pubkey());

    delegate_auctioneer(
        &mut context,
        ahkey,
        &ah_auth,
        auctioneer_authority.pubkey(),
        auctioneer_pda,
        auctioneer_pda_bump,
        default_scopes(),
    )
    .await
    .unwrap();

    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), 2000000000)
        .await
        .unwrap();
    let (acc, deposit_tx) = auction_deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer,
        auctioneer_authority.pubkey(),
        1000000000,
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
    assert_eq!(escrow.lamports, 1000000000);
}

#[tokio::test]
async fn auction_deposit_missing_scope_fails() {
    let mut context = auction_house_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, ah_auth) = existing_auction_house_test_context(&mut context)
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

    // Delegate external auctioneer authority.
    let auctioneer_authority = Keypair::new();
    let (auctioneer_pda, auctioneer_pda_bump) =
        find_auctioneer_pda(&ahkey, &auctioneer_authority.pubkey());

    // Missing Deposit scope, so tx should fail.
    let scopes = vec![];

    delegate_auctioneer(
        &mut context,
        ahkey,
        &ah_auth,
        auctioneer_authority.pubkey(),
        auctioneer_pda,
        auctioneer_pda_bump,
        scopes.clone(),
    )
    .await
    .unwrap();

    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), 2000000000)
        .await
        .unwrap();
    let (_, deposit_tx) = auction_deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer,
        auctioneer_authority.pubkey(),
        1000000000,
    );

    let error = context
        .banks_client
        .process_transaction(deposit_tx)
        .await
        .unwrap_err();
    assert_error!(error, MISSING_AUCTIONEER_SCOPE);
}

#[tokio::test]
async fn auction_deposit_no_delegate_fails() {
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

    // Delegate external auctioneer authority.
    let auctioneer_authority = Keypair::new();

    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), 2000000000)
        .await
        .unwrap();
    let (_acc, deposit_tx) = auction_deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer,
        auctioneer_authority.pubkey(),
        1000000000,
    );

    let error = context
        .banks_client
        .process_transaction(deposit_tx)
        .await
        .unwrap_err();

    assert_error!(error, NO_AUCTIONEER_PROGRAM_SET);
}
