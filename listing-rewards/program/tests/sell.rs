#![cfg(feature = "test-bpf")]

pub mod listing_rewards_test;

use anchor_client::solana_sdk::{pubkey::Pubkey, signature::Signer, transaction::Transaction};
use mpl_auction_house::{
    pda::{
        find_auction_house_address, find_auctioneer_trade_state_address, find_trade_state_address,
    },
    AuthorityScope,
};
use mpl_listing_rewards::{
    pda::{find_listing_address, find_reward_center_address, find_rewardable_collection_address},
    reward_center,
};

use solana_program_test::*;
use std::str::FromStr;

use mpl_testing_utils::{solana::airdrop, utils::Metadata};

use spl_associated_token_account::get_associated_token_address;
use spl_token::native_mint;

#[tokio::test]
async fn sell_success() {
    let program = listing_rewards_test::setup_program();
    let mut context = program.start_with_context().await;

    let wallet = context.payer.pubkey();
    let mint = native_mint::id();
    let collection = Pubkey::from_str("Cehzo7ugAvuYcTst9HF24ackLxnrpDkzHFajj17FuyUR").unwrap();

    let test_metadata = Metadata::new();
    let owner_pubkey = &test_metadata.token.pubkey();
    airdrop(&mut context, owner_pubkey, listing_rewards_test::TEN_SOL)
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

    let (auction_house, _) = find_auction_house_address(&wallet, &mint);
    let (reward_center, _) = find_reward_center_address(&auction_house);
    let (rewardable_collection, _) =
        find_rewardable_collection_address(&reward_center, &collection);
    let (listing, _) = find_listing_address(
        owner_pubkey,
        &test_metadata.mint.pubkey(),
        &rewardable_collection,
    );

    let reward_center_params = reward_center::CreateRewardCenterParams {
        collection_oracle: None,
        listing_reward_rules: reward_center::ListingRewardRules {
            warmup_minutes: 2 * 24 * 60,
            reward_payout: 1000,
        },
    };

    let create_auction_house_accounts = mpl_auction_house_sdk::CreateAuctionHouseAccounts {
        treasury_mint: mint,
        payer: wallet,
        authority: wallet,
        fee_withdrawal_destination: wallet,
        treasury_withdrawal_destination: wallet,
        treasury_withdrawal_destination_owner: wallet,
    };
    let create_auction_house_data = mpl_auction_house_sdk::CreateAuctionHouseData {
        seller_fee_basis_points: 100,
        requires_sign_off: false,
        can_change_sale_price: false,
    };

    let create_auction_house_ix = mpl_auction_house_sdk::create_auction_house(
        create_auction_house_accounts,
        create_auction_house_data,
    );

    let create_reward_center_ix = mpl_listing_rewards_sdk::create_reward_center(
        wallet,
        mint,
        auction_house,
        reward_center_params,
    );

    let create_rewardable_collection_ix = mpl_listing_rewards_sdk::create_rewardable_collection(
        wallet,
        auction_house,
        reward_center,
        collection,
    );

    let delegate_auctioneer_accounts = mpl_auction_house_sdk::DelegateAuctioneerAccounts {
        auction_house,
        authority: wallet,
        auctioneer_authority: reward_center,
    };

    let delegate_auctioneer_data = mpl_auction_house_sdk::DelegateAuctioneerData {
        scopes: vec![
            AuthorityScope::Deposit,
            AuthorityScope::Buy,
            AuthorityScope::PublicBuy,
            AuthorityScope::ExecuteSale,
            AuthorityScope::Sell,
            AuthorityScope::Cancel,
            AuthorityScope::Withdraw,
        ],
    };

    let delegate_auctioneer_ix = mpl_auction_house_sdk::delegate_auctioneer(
        delegate_auctioneer_accounts,
        delegate_auctioneer_data,
    );

    let token_account = get_associated_token_address(owner_pubkey, &test_metadata.mint.pubkey());

    let (seller_trade_state, trade_state_bump) = find_auctioneer_trade_state_address(
        owner_pubkey,
        &auction_house,
        &token_account,
        &mint,
        &test_metadata.mint.pubkey(),
        1,
    );

    let (free_seller_trade_state, free_trade_state_bump) = find_trade_state_address(
        owner_pubkey,
        &auction_house,
        &token_account,
        &mint,
        &test_metadata.mint.pubkey(),
        0,
        1,
    );

    let sell_accounts = mpl_listing_rewards_sdk::SellAccounts {
        wallet: *owner_pubkey,
        listing,
        reward_center,
        rewardable_collection,
        token_account,
        metadata: test_metadata.pubkey,
        authority: wallet,
        auction_house,
        seller_trade_state,
        free_seller_trade_state,
    };

    let sell_params = mpl_listing_rewards_sdk::SellData {
        price: listing_rewards_test::ONE_SOL,
        token_size: 1,
        collection,
        trade_state_bump,
        free_trade_state_bump,
    };

    let sell_ix = mpl_listing_rewards_sdk::sell(sell_accounts, sell_params);

    let tx = Transaction::new_signed_with_payer(
        &[
            create_auction_house_ix,
            create_reward_center_ix,
            create_rewardable_collection_ix,
            delegate_auctioneer_ix,
        ],
        Some(&wallet),
        &[&context.payer],
        context.last_blockhash,
    );

    let tx_response = context.banks_client.process_transaction(tx).await;

    assert!(tx_response.is_ok());

    let tx = Transaction::new_signed_with_payer(
        &[sell_ix],
        Some(owner_pubkey),
        &[&test_metadata.token],
        context.last_blockhash,
    );

    let tx_response = context.banks_client.process_transaction(tx).await;

    assert!(tx_response.is_ok());

    ()
}
