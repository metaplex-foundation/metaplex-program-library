#![cfg(feature = "test-bpf")]
pub mod common;
pub mod utils;

use common::*;
use utils::{
    helpers::{assert_scopes_eq, default_scopes},
    setup_functions::*,
};

#[tokio::test]
async fn overwrite_scopes() {
    /*
    Test overwriting scopes for an auction house.

    Create an auction house, delegate it to a new auction house authority, with all available scopes.
    Update it to have only one scope and ensure that the auction house is not able to execute handlers that require old scopes.
     */

    // Setup program test context and a new auction house.
    let mut context = auction_house_program_test().start_with_context().await;
    let (ah, ahkey, ah_auth) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();

    // Create a NFT.
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

    // Call `delegate_auctioneer` with the auction house authority and a new auctioneer program.
    let auctioneer_authority = Keypair::new();
    let auctioneer_authority_pubkey = auctioneer_authority.pubkey();

    let (auctioneer_pda, _) = find_auctioneer_pda(&ahkey, &auctioneer_authority_pubkey);

    let scopes = default_scopes();

    delegate_auctioneer(
        &mut context,
        ahkey,
        &ah_auth,
        auctioneer_authority_pubkey,
        auctioneer_pda,
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

    assert!(!ah.has_auctioneer);
    assert!(new_ah.has_auctioneer);

    assert_eq!(auctioneer_authority_pubkey, auctioneer.auctioneer_authority);
    assert_eq!(ahkey, auctioneer.auction_house);
    assert_scopes_eq(scopes, auctioneer.scopes);

    // Try to deposit funds to the auction house. This should succeed.
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

    context
        .banks_client
        .process_transaction(deposit_tx)
        .await
        .unwrap();

    // Update the auction house to have only one scope.
    let new_scopes = vec![AuthorityScope::Buy];

    update_auctioneer(
        &mut context,
        ahkey,
        &ah_auth,
        auctioneer_authority_pubkey,
        auctioneer_pda,
        new_scopes.clone(),
    )
    .await
    .unwrap();

    // Try to deposit funds to the auction house; should fail with missing scope error.
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
