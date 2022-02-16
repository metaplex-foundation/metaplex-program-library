#![cfg(feature = "test-bpf")]
mod utils;
use anchor_lang::{InstructionData, ToAccountMetas};

use mpl_testing_utils::solana::airdrop;
use mpl_testing_utils::utils::Metadata;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signer;
use solana_sdk::{instruction::Instruction, transaction::Transaction};

use mpl_auction_house::ErrorCode;
use spl_associated_token_account::get_associated_token_address;
use spl_token;

use crate::utils::setup_functions::buy;
use utils::setup_functions::{
    auction_house_program_test, existing_auction_house_test_context, sell,
};

#[tokio::test]
async fn cancel_listing() {
    let mut context = auction_house_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, _) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();
    airdrop(
        &mut context,
        &test_metadata.token.pubkey(),
        100_000_000_000_000,
    )
    .await
    .unwrap();
    test_metadata
        .create(
            &mut context,
            "Tests".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            None,
            10,
            false,
        )
        .await
        .unwrap();
    context.warp_to_slot(100).unwrap();
    // Derive Auction House Key
    let (acc, sell_tx) = sell(&mut context, &ahkey, &ah, &test_metadata, 10);
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();
    let token =
        get_associated_token_address(&test_metadata.token.pubkey(), &test_metadata.mint.pubkey());
    let accounts = mpl_auction_house::accounts::Cancel {
        auction_house: ahkey,
        wallet: test_metadata.token.pubkey(),
        token_account: token,
        authority: ah.authority,
        trade_state: acc.seller_trade_state,
        token_program: spl_token::id(),
        token_mint: test_metadata.mint.pubkey(),
        auction_house_fee_account: ah.auction_house_fee_account,
    }
    .to_account_metas(None);
    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::Cancel {
            buyer_price: 10,
            token_size: 1,
        }
        .data(),
        accounts,
    };
    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&test_metadata.token.pubkey()),
        &[&test_metadata.token],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();
}

#[tokio::test]
async fn cancel_bid() {
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
            "Tests".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            None,
            10,
            false,
        )
        .await
        .unwrap();
    context.warp_to_slot(100).unwrap();
    let buyer = Keypair::new();
    // Derive Auction House Key
    let price = 1000000000;
    airdrop(&mut context, &buyer.pubkey(), 2000000000)
        .await
        .unwrap();
    let (acc, buy_tx) = buy(&mut context, &ahkey, &ah, &test_metadata, &test_metadata.token.pubkey(), &buyer, price);

    context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap();
    let accounts = mpl_auction_house::accounts::Cancel {
        auction_house: ahkey,
        wallet: buyer.pubkey(),
        token_account: acc.token_account,
        authority: ah.authority,
        trade_state: acc.buyer_trade_state,
        token_program: spl_token::id(),
        token_mint: test_metadata.mint.pubkey(),
        auction_house_fee_account: ah.auction_house_fee_account,
    }
    .to_account_metas(None);
    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::Cancel {
            buyer_price: price,
            token_size: 1,
        }
        .data(),
        accounts,
    };
    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&buyer.pubkey()),
        &[&buyer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();
}
