#![cfg(feature = "test-bpf")]

pub mod listing_rewards_test;

use anchor_client::solana_sdk::{pubkey::Pubkey, signature::Signer, transaction::Transaction};
use mpl_auction_house::pda::find_auction_house_address;
use mpl_listing_rewards::{pda::find_reward_center_address, reward_center, state};
use solana_program_test::*;
use std::{println, str::FromStr};

use spl_token::native_mint;

#[tokio::test]
async fn recreate_rewardable_collection_success() {
    let program = listing_rewards_test::setup_program();
    let mut context = program.start_with_context().await;

    let wallet = context.payer.pubkey();
    let mint = native_mint::id();

    let collection = Pubkey::from_str(listing_rewards_test::TEST_COLLECTION).unwrap();

    let (auction_house, _) = find_auction_house_address(&wallet, &mint);
    let (reward_center, _) = find_reward_center_address(&auction_house);

    let reward_center_params = reward_center::CreateRewardCenterParams {
        collection_oracle: None,
        listing_reward_rules: state::ListingRewardRules {
            warmup_seconds: 2 * 24 * 60 * 60,
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

    let delete_rewardable_collection_ix = mpl_listing_rewards_sdk::delete_rewardable_collection(
        wallet,
        auction_house,
        reward_center,
        collection,
    );

    let recreate_rewardable_collection_ix = mpl_listing_rewards_sdk::create_rewardable_collection(
        wallet,
        auction_house,
        reward_center,
        collection,
    );

    let tx = Transaction::new_signed_with_payer(
        &[
            create_auction_house_ix,
            create_reward_center_ix,
            create_rewardable_collection_ix,
            delete_rewardable_collection_ix,
            recreate_rewardable_collection_ix,
        ],
        Some(&wallet),
        &[&context.payer],
        context.last_blockhash,
    );

    let tx_response = context.banks_client.process_transaction(tx).await;

    println!("{:?}", tx_response);

    assert!(tx_response.is_ok());

    ()
}
