#![cfg(feature = "test-bpf")]

pub mod listing_rewards_test;

use anchor_client::solana_sdk::{signature::Signer, transaction::Transaction};
use mpl_auction_house::pda::find_auction_house_address;
use mpl_listing_rewards::{reward_center, state};
use solana_program_test::*;

use solana_sdk::signature::Keypair;
use spl_token::native_mint;

#[tokio::test]
async fn create_reward_center_success() {
    let program = listing_rewards_test::setup_program();
    let mut context = program.start_with_context().await;

    let wallet = context.payer.pubkey();
    let mint = native_mint::id();

    let (auction_house, _) = find_auction_house_address(&wallet, &mint);

    let reward_center_params = reward_center::create::CreateRewardCenterParams {
        collection_oracle: None,
        listing_reward_rules: state::ListingRewardRules {
            warmup_seconds: 2 * 24 * 60 * 60,
            reward_payout: 1000,
            seller_reward_payout_basis_points: 1000,
            payout_divider: 5,
        },
    };

    let test_collection_oracle = Keypair::new().pubkey();

    let edit_reward_center_params = reward_center::edit::EditRewardCenterParams {
        collection_oracle: Some(test_collection_oracle),
        listing_reward_rules: state::ListingRewardRules {
            warmup_seconds: 10 * 24 * 60 * 60,
            reward_payout: 2000,
            seller_reward_payout_basis_points: 2000,
            payout_divider: 10,
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

    let create_auction_house_id = mpl_auction_house_sdk::create_auction_house(
        create_auction_house_accounts,
        create_auction_house_data,
    );

    let create_reward_center_ix = mpl_listing_rewards_sdk::create_reward_center(
        wallet,
        mint,
        auction_house,
        reward_center_params,
    );

    let edit_reward_center_ix = mpl_listing_rewards_sdk::edit_reward_center(
        wallet,
        auction_house,
        edit_reward_center_params,
    );

    let tx = Transaction::new_signed_with_payer(
        &[
            create_auction_house_id,
            create_reward_center_ix,
            edit_reward_center_ix,
        ],
        Some(&wallet),
        &[&context.payer],
        context.last_blockhash,
    );

    let tx_response = context.banks_client.process_transaction(tx).await;

    assert!(tx_response.is_ok());

    ()
}
