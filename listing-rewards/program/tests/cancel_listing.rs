#![cfg(feature = "test-bpf")]

pub mod listing_rewards_test;

use listing_rewards_test::fixtures::metadata;

use anchor_client::solana_sdk::{pubkey::Pubkey, signature::Signer, transaction::Transaction};
use mpl_auction_house::{
    pda::{
        find_auction_house_address, find_auctioneer_trade_state_address, find_trade_state_address,
    },
    AuthorityScope,
};
use mpl_listing_rewards::{
    pda::{find_listing_address, find_reward_center_address, find_rewardable_collection_address},
    reward_center, state,
};

use mpl_listing_rewards_sdk::{
    accounts::{CancelListingAccounts, *},
    args::*,
    *,
};

use solana_program_test::*;
use std::str::FromStr;

use mpl_token_metadata::state::Collection;

use spl_associated_token_account::get_associated_token_address;
use spl_token::native_mint;

#[tokio::test]
async fn cancel_listing_success() {
    let program = listing_rewards_test::setup_program();
    let mut context = program.start_with_context().await;

    let wallet = context.payer.pubkey();
    let mint = native_mint::id();
    let collection = Pubkey::from_str(listing_rewards_test::TEST_COLLECTION).unwrap();

    let metadata = metadata::create(
        &mut context,
        metadata::Params {
            name: "Test",
            symbol: "TST",
            uri: "https://nfts.exp.com/1.json",
            creators: None,
            seller_fee_basis_points: 10,
            is_mutable: false,
            collection: Some(Collection {
                verified: false,
                key: collection,
            }),
            uses: None,
        },
        None,
    )
    .await;

    let metadata_owner = metadata.token;
    let metadata_address = metadata.pubkey;
    let metadata_owner_address = metadata_owner.pubkey();
    let metadata_mint_address = metadata.mint.pubkey();

    let (auction_house, _) = find_auction_house_address(&wallet, &mint);
    let (reward_center, _) = find_reward_center_address(&auction_house);
    let (rewardable_collection, _) =
        find_rewardable_collection_address(&reward_center, &collection);
    let (listing, _) = find_listing_address(
        &metadata_owner_address,
        &metadata_address,
        &rewardable_collection,
    );

    let reward_center_params = reward_center::create::CreateRewardCenterParams {
        collection_oracle: None,
        listing_reward_rules: state::ListingRewardRules {
            warmup_seconds: 2 * 24 * 60 * 60,
            reward_payout: 1000,
            seller_reward_payout_basis_points: 1000,
            payout_divider: 5,
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

    let token_account =
        get_associated_token_address(&metadata_owner_address, &metadata_mint_address);

    let (seller_trade_state, trade_state_bump) = find_auctioneer_trade_state_address(
        &metadata_owner_address,
        &auction_house,
        &token_account,
        &mint,
        &metadata_mint_address,
        1,
    );

    let (free_seller_trade_state, free_trade_state_bump) = find_trade_state_address(
        &metadata_owner_address,
        &auction_house,
        &token_account,
        &mint,
        &metadata_mint_address,
        0,
        1,
    );

    let create_listing_accounts = CreateListingAccounts {
        wallet: metadata_owner.pubkey(),
        listing,
        reward_center,
        rewardable_collection,
        token_account,
        metadata: metadata.pubkey,
        authority: wallet,
        auction_house,
        seller_trade_state,
        free_seller_trade_state,
    };

    let create_listing_params = CreateListingData {
        price: listing_rewards_test::ONE_SOL,
        token_size: 1,
        trade_state_bump,
        free_trade_state_bump,
    };

    let create_listing_ix = create_listing(create_listing_accounts, create_listing_params);

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
        &[create_listing_ix],
        Some(&metadata_owner_address),
        &[&metadata_owner],
        context.last_blockhash,
    );

    let tx_response = context.banks_client.process_transaction(tx).await;

    assert!(tx_response.is_ok());

    // CANCEL LISTING TEST

    let cancel_listing_accounts = CancelListingAccounts {
        wallet: metadata_owner_address,
        listing,
        reward_center,
        rewardable_collection,
        token_account,
        metadata: metadata_address,
        authority: wallet,
        auction_house,
        treasury_mint: mint,
        token_mint: metadata_mint_address,
    };

    let cancel_listing_params = CancelListingData {
        price: u64::MAX,
        token_size: 1,
    };

    let cancel_listing_ix = cancel_listing(cancel_listing_accounts, cancel_listing_params);

    let tx = Transaction::new_signed_with_payer(
        &[cancel_listing_ix],
        Some(&metadata_owner_address),
        &[&metadata_owner],
        context.last_blockhash,
    );

    let tx_response = context.banks_client.process_transaction(tx).await;

    assert!(tx_response.is_ok());

    ()
}
