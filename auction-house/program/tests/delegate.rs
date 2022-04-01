#![cfg(feature = "test-bpf")]
pub mod utils;

use anchor_lang::AnchorDeserialize;
use mpl_auction_house::{pda::find_auctioneer_pda, AuctionHouse, Auctioneer, AuthorityScope};
use mpl_testing_utils::{assert_error, solana::airdrop};
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, signature::Keypair, signer::Signer,
    transaction::TransactionError, transport::TransportError,
};
use std::assert_eq;
use utils::setup_functions::*;

#[tokio::test]
async fn delegate_success() {
    // **ARRANGE**

    // Setup program test context and a new auction house.
    let mut context = auction_house_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, ah_authority) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();

    // **ACT**

    // Call `delegate_auctioneer` with the auction house authority and a new auctioneer program.
    let auctioneer_authority = Keypair::new();
    let auctioneer_authority_pubkey = auctioneer_authority.pubkey();

    let (auctioneer_pda, auctioneer_pda_bump) =
        find_auctioneer_pda(&ahkey, &auctioneer_authority_pubkey);

    let scopes = vec![
        AuthorityScope::Buy,
        AuthorityScope::PublicBuy,
        AuthorityScope::ExecuteSale,
        AuthorityScope::Sell,
        AuthorityScope::Cancel,
        AuthorityScope::Withdraw,
    ];

    delegate(
        &mut context,
        ahkey,
        ah_authority,
        auctioneer_authority_pubkey,
        auctioneer_pda,
        auctioneer_pda_bump,
        scopes.clone(),
    )
    .await
    .unwrap();

    let new_ah_account = context
        .banks_client
        .get_account(ahkey)
        .await
        .expect("Error getting new auction house account")
        .expect("Auction House empty");
    let new_ah = AuctionHouse::deserialize(&mut new_ah_account.data[8..].as_ref())
        .expect("Failed to deserialize Auction House data");

    let auctioneer_pda_account = context
        .banks_client
        .get_account(auctioneer_pda)
        .await
        .expect("Error getting new auction house account")
        .expect("Auction House empty");
    let auctioneer = Auctioneer::deserialize(&mut auctioneer_pda_account.data[8..].as_ref())
        .expect("Failed to deserialize Auction House data");

    // **ASSERT**
    assert!(!ah.has_auctioneer);
    assert!(new_ah.has_auctioneer);

    assert_eq!(auctioneer_authority_pubkey, auctioneer.authority);
    assert_eq!(ahkey, auctioneer.auction_house);
    assert_eq!(scopes, auctioneer.scopes);
}

#[tokio::test]
async fn incorrect_authority_fails() {
    // Setup program test context and a new auction house.
    let mut context = auction_house_program_test().start_with_context().await;
    // Payer Wallet
    let (_, ahkey, _) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();

    let invalid_authority = Keypair::new();
    airdrop(&mut context, &invalid_authority.pubkey(), 10_000_000_000)
        .await
        .expect("Failed to airdrop to invalid authority");

    // Call `delegate_auctioneer` with the auction house authority and a new auctioneer program.
    let auctioneer_authority = Keypair::new();
    let auctioneer_authority_pubkey = auctioneer_authority.pubkey();

    let (auctioneer_pda, auctioneer_pda_bump) =
        find_auctioneer_pda(&ahkey, &auctioneer_authority_pubkey);

    let scopes = vec![
        AuthorityScope::Buy,
        AuthorityScope::PublicBuy,
        AuthorityScope::ExecuteSale,
        AuthorityScope::Sell,
        AuthorityScope::Cancel,
        AuthorityScope::Withdraw,
    ];

    let err = delegate(
        &mut context,
        ahkey,
        invalid_authority,
        auctioneer_authority_pubkey,
        auctioneer_pda,
        auctioneer_pda_bump,
        scopes.clone(),
    )
    .await
    .unwrap_err();
    println!("{:?}", err.to_string());

    assert_error!(err, 2001);
}

#[tokio::test]
async fn too_many_scopes() {
    // Setup program test context and a new auction house.
    let mut context = auction_house_program_test().start_with_context().await;
    // Payer Wallet
    let (_, ahkey, ah_auth) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();

    // Call `delegate_auctioneer` with the auction house authority and a new auctioneer program.
    let auctioneer_authority = Keypair::new();
    let auctioneer_authority_pubkey = auctioneer_authority.pubkey();

    let (auctioneer_pda, auctioneer_pda_bump) =
        find_auctioneer_pda(&ahkey, &auctioneer_authority_pubkey);

    let scopes = vec![
        AuthorityScope::Buy,
        AuthorityScope::PublicBuy,
        AuthorityScope::ExecuteSale,
        AuthorityScope::Sell,
        AuthorityScope::Cancel,
        AuthorityScope::Withdraw,
        AuthorityScope::Buy,
    ];

    let err = delegate(
        &mut context,
        ahkey,
        ah_auth,
        auctioneer_authority_pubkey,
        auctioneer_pda,
        auctioneer_pda_bump,
        scopes.clone(),
    )
    .await
    .unwrap_err();
    println!("{:?}", err.to_string());

    assert_error!(err, 6032);
}
