#![cfg(feature = "test-bpf")]
pub mod common;
pub mod utils;

use common::*;
use utils::{helpers::*, setup_functions::*};

use mpl_testing_utils::{solana::airdrop, utils::Metadata};
use solana_sdk::signer::Signer;
use std::assert_eq;

#[tokio::test]
async fn withdraw_success() {
    // Setup program test context and a new auction house.
    let mut context = auction_house_program_test().start_with_context().await;

    // Payer Wallet
    let (ah, ahkey, _) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();

    // Setup NFT metadata and owner keypair.
    let test_metadata = Metadata::new();
    let owner_pubkey = &test_metadata.token.pubkey();

    let airdrop_amount = 2_000_000_000;
    // Airdrop owner with some SOL.
    airdrop(&mut context, owner_pubkey, airdrop_amount)
        .await
        .unwrap();

    let escrow_payment_account_data_len = 0;
    let rent = context.banks_client.get_rent().await.unwrap();
    let rent_exempt_min: u64 = rent.minimum_balance(escrow_payment_account_data_len);

    let sale_price = 1_000_000_000;

    // Create NFT.
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

    // Create a new account for the buyer, airdrop to the wallet and deposit to an AH escrow account.
    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), airdrop_amount)
        .await
        .unwrap();

    let (acc, deposit_tx) = deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer,
        sale_price,
    );
    context
        .banks_client
        .process_transaction(deposit_tx)
        .await
        .unwrap();

    let escrow_balance_before_withdraw = context
        .banks_client
        .get_account(acc.escrow_payment_account)
        .await
        .expect("Error Getting Escrow")
        .expect("Trade State Escrow")
        .lamports;

    let (_, withdraw_tx) = withdraw(
        &mut context,
        &buyer,
        &ahkey,
        &ah,
        &test_metadata,
        sale_price,
        sale_price,
    );
    context
        .banks_client
        .process_transaction(withdraw_tx)
        .await
        .unwrap();

    let escrow_balance_after_withdraw = context
        .banks_client
        .get_account(acc.escrow_payment_account)
        .await
        .expect("Error Getting Escrow")
        .expect("Trade State Escrow")
        .lamports;

    assert_eq!(sale_price + rent_exempt_min, escrow_balance_before_withdraw);
    assert_eq!(rent_exempt_min, escrow_balance_after_withdraw);
}

#[tokio::test]
async fn withdraw_insufficient_funds_fails() {
    // Setup program test context and a new auction house.
    let mut context = auction_house_program_test().start_with_context().await;

    // Payer Wallet
    let (ah, ahkey, _) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();

    // Setup NFT metadata and owner keypair.
    let test_metadata = Metadata::new();
    let owner_pubkey = &test_metadata.token.pubkey();

    let airdrop_amount = 2_000_000_000;
    // Airdrop owner with some SOL.
    airdrop(&mut context, owner_pubkey, airdrop_amount)
        .await
        .unwrap();

    let escrow_payment_account_data_len = 0;
    let rent = context.banks_client.get_rent().await.unwrap();
    let rent_exempt_min: u64 = rent.minimum_balance(escrow_payment_account_data_len);

    let sale_price = 1_000_000_000;

    // Create NFT.
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

    // Create a new account for the buyer, airdrop to the wallet and deposit to an AH escrow account.
    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), airdrop_amount)
        .await
        .unwrap();

    let (acc, deposit_tx) = deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer,
        sale_price,
    );
    context
        .banks_client
        .process_transaction(deposit_tx)
        .await
        .unwrap();

    let escrow_balance_before_withdraw = context
        .banks_client
        .get_account(acc.escrow_payment_account)
        .await
        .expect("Error Getting Escrow")
        .expect("Trade State Escrow")
        .lamports;

    let (_, withdraw_tx) = withdraw(
        &mut context,
        &buyer,
        &ahkey,
        &ah,
        &test_metadata,
        sale_price,
        sale_price + rent_exempt_min + 1,
    );

    let error = context
        .banks_client
        .process_transaction(withdraw_tx)
        .await
        .unwrap_err();

    assert_error!(error, INSUFFICIENT_FUNDS);
    assert_eq!(sale_price + rent_exempt_min, escrow_balance_before_withdraw);
}

#[tokio::test]
async fn withdraw_extra_and_adjusted_for_rent_shortfall_success() {
    // Setup program test context and a new auction house.
    let mut context = auction_house_program_test().start_with_context().await;

    // Payer Wallet
    let (ah, ahkey, _) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();

    // Setup NFT metadata and owner keypair.
    let test_metadata = Metadata::new();
    let owner_pubkey = &test_metadata.token.pubkey();

    let airdrop_amount = 2_000_000_000;
    // Airdrop owner with some SOL.
    airdrop(&mut context, owner_pubkey, airdrop_amount)
        .await
        .unwrap();

    let escrow_payment_account_data_len = 0;
    let rent = context.banks_client.get_rent().await.unwrap();
    let rent_exempt_min: u64 = rent.minimum_balance(escrow_payment_account_data_len);

    let sale_price = 1_000_000_000;

    // Create NFT.
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

    // Create a new account for the buyer, airdrop to the wallet and deposit to an AH escrow account.
    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), airdrop_amount)
        .await
        .unwrap();

    let (acc, deposit_tx) = deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer,
        sale_price,
    );
    context
        .banks_client
        .process_transaction(deposit_tx)
        .await
        .unwrap();

    let escrow_balance_before_withdraw = context
        .banks_client
        .get_account(acc.escrow_payment_account)
        .await
        .expect("Error Getting Escrow")
        .expect("Trade State Escrow")
        .lamports;

    let (_, withdraw_tx) = withdraw(
        &mut context,
        &buyer,
        &ahkey,
        &ah,
        &test_metadata,
        sale_price,
        sale_price + rent_exempt_min,
    );
    context
        .banks_client
        .process_transaction(withdraw_tx)
        .await
        .unwrap();

    let escrow_balance_after_withdraw = context
        .banks_client
        .get_account(acc.escrow_payment_account)
        .await
        .expect("Error Getting Escrow")
        .expect("Trade State Escrow")
        .lamports;

    assert_eq!(sale_price + rent_exempt_min, escrow_balance_before_withdraw);
    assert_eq!(rent_exempt_min, escrow_balance_after_withdraw);
}

#[tokio::test]
async fn auctioneer_withdraw_success() {
    // Setup program test context and a new auction house.
    let mut context = auction_house_program_test().start_with_context().await;

    // Payer Wallet
    let (ah, ahkey, ah_auth) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();

    // Setup NFT metadata and owner keypair.
    let test_metadata = Metadata::new();
    let owner_pubkey = &test_metadata.token.pubkey();

    let airdrop_amount = 10_000_000_000;
    // Airdrop owner with some SOL.
    airdrop(&mut context, owner_pubkey, airdrop_amount)
        .await
        .unwrap();

    let escrow_payment_account_data_len = 0;
    let rent = context.banks_client.get_rent().await.unwrap();
    let rent_exempt_min: u64 = rent.minimum_balance(escrow_payment_account_data_len);

    let sale_price = 1_000_000_000;

    // Create NFT.
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

    // Create a new account for the buyer, airdrop to the wallet and deposit to an AH escrow account.
    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), airdrop_amount)
        .await
        .unwrap();

    // Delegate external auctioneer authority.
    let auctioneer_authority = Keypair::new();
    let (auctioneer_pda, _) = find_auctioneer_pda(&ahkey, &auctioneer_authority.pubkey());

    delegate_auctioneer(
        &mut context,
        ahkey,
        &ah_auth,
        auctioneer_authority.pubkey(),
        auctioneer_pda,
        default_scopes(),
    )
    .await
    .unwrap();

    let (acc, deposit_tx) = auctioneer_deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer,
        &auctioneer_authority,
        sale_price,
    );
    context
        .banks_client
        .process_transaction(deposit_tx)
        .await
        .unwrap();

    let escrow_balance_before_withdraw = context
        .banks_client
        .get_account(acc.escrow_payment_account)
        .await
        .expect("Error Getting Escrow")
        .expect("Trade State Escrow")
        .lamports;

    let (_, withdraw_tx) = auctioneer_withdraw(
        &mut context,
        &buyer,
        &ahkey,
        &ah,
        &test_metadata,
        &auctioneer_authority,
        sale_price,
        sale_price,
    );
    context
        .banks_client
        .process_transaction(withdraw_tx)
        .await
        .unwrap();

    let escrow_balance_after_withdraw = context
        .banks_client
        .get_account(acc.escrow_payment_account)
        .await
        .expect("Error Getting Escrow")
        .expect("Trade State Escrow")
        .lamports;

    assert_eq!(sale_price + rent_exempt_min, escrow_balance_before_withdraw);
    assert_eq!(rent_exempt_min, escrow_balance_after_withdraw);
}

#[tokio::test]
async fn auctioneer_withdraw_missing_scope_fails() {
    // Setup program test context and a new auction house.
    let mut context = auction_house_program_test().start_with_context().await;

    // Payer Wallet
    let (ah, ahkey, ah_auth) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();

    // Setup NFT metadata and owner keypair.
    let test_metadata = Metadata::new();
    let owner_pubkey = &test_metadata.token.pubkey();

    // Airdrop owner with some SOL.
    airdrop(&mut context, owner_pubkey, 10_000_000_000)
        .await
        .unwrap();

    let airdrop_amount = 10_000_000_000;
    let sale_price = 1_000_000_000;
    let withdraw_price = sale_price / 2;

    // Create NFT.
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

    // Create a new account for the buyer, airdrop to the wallet and deposit to an AH escrow account.
    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), airdrop_amount)
        .await
        .unwrap();

    // Delegate external auctioneer authority.
    let auctioneer_authority = Keypair::new();
    let (auctioneer_pda, _) = find_auctioneer_pda(&ahkey, &auctioneer_authority.pubkey());

    let scopes = vec![AuthorityScope::Deposit];

    delegate_auctioneer(
        &mut context,
        ahkey,
        &ah_auth,
        auctioneer_authority.pubkey(),
        auctioneer_pda,
        scopes.clone(),
    )
    .await
    .unwrap();

    let (_, deposit_tx) = auctioneer_deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer,
        &auctioneer_authority,
        sale_price,
    );
    context
        .banks_client
        .process_transaction(deposit_tx)
        .await
        .unwrap();

    let (_, withdraw_tx) = auctioneer_withdraw(
        &mut context,
        &buyer,
        &ahkey,
        &ah,
        &test_metadata,
        &auctioneer_authority,
        sale_price,
        withdraw_price,
    );
    let error = context
        .banks_client
        .process_transaction(withdraw_tx)
        .await
        .unwrap_err();
    assert_error!(error, MISSING_AUCTIONEER_SCOPE);
}

#[tokio::test]
async fn auctioneer_withdraw_no_delegate_fails() {
    // Setup program test context and a new auction house.
    let mut context = auction_house_program_test().start_with_context().await;

    // Payer Wallet
    let (ah, ahkey, _) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();

    // Setup NFT metadata and owner keypair.
    let test_metadata = Metadata::new();
    let owner_pubkey = &test_metadata.token.pubkey();

    // Airdrop owner with some SOL.
    airdrop(&mut context, owner_pubkey, 10_000_000_000)
        .await
        .unwrap();

    let airdrop_amount = 10_000_000_000;
    let sale_price = 1_000_000_000;
    let withdraw_price = sale_price / 2;

    // Create NFT.
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

    // Create a new account for the buyer, airdrop to the wallet and deposit to an AH escrow account.
    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), airdrop_amount)
        .await
        .unwrap();

    // Delegate external auctioneer authority.
    let auctioneer_authority = Keypair::new();

    let (_acc, deposit_tx) = deposit(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &buyer,
        sale_price,
    );
    context
        .banks_client
        .process_transaction(deposit_tx)
        .await
        .unwrap();

    let (_, withdraw_tx) = auctioneer_withdraw(
        &mut context,
        &buyer,
        &ahkey,
        &ah,
        &test_metadata,
        &auctioneer_authority,
        sale_price,
        withdraw_price,
    );
    let error = context
        .banks_client
        .process_transaction(withdraw_tx)
        .await
        .unwrap_err();

    assert_error!(error, INVALID_SEEDS);
}
