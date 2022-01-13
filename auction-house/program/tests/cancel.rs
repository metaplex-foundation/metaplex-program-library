#![cfg(feature = "test-bpf")]
mod utils;
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};

use mpl_auction_house::{pda::*, AuctionHouse};
use mpl_testing_utils::solana::{airdrop, create_associated_token_account, create_mint, transfer};
use mpl_testing_utils::{assert_error, utils::Metadata};
use solana_program_test::*;
use solana_sdk::{
    instruction::{Instruction, InstructionError},
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};
use solana_sdk::{
    signature::{Keypair, Signer},
    sysvar,
};

use spl_associated_token_account::get_associated_token_address;
use spl_token;
use std::assert_eq;
use utils::setup_functions::{
    self, auction_house_program_test, existing_auction_house_test_context,
};
#[tokio::test]
async fn cancel() {
    let mut context = auction_house_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();
    airdrop(
        &mut context,
        &test_metadata.token.pubkey(),
        10_000_000_000_000,
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
    let (st, fst) = setup_functions::sell(&mut context, &ahkey, &ah, &test_metadata, 1)
        .await
        .unwrap();
    let token =
        get_associated_token_address(&test_metadata.token.pubkey(), &test_metadata.mint.pubkey());
    let accounts = mpl_auction_house::accounts::Cancel {
        auction_house: ahkey,
        wallet: test_metadata.token.pubkey(),
        token_account: token,
        authority: ah.authority,
        trade_state: st,
        token_program: spl_token::id(),
        token_mint: test_metadata.mint.pubkey(),
        auction_house_fee_account: ah.auction_house_fee_account,
    }
    .to_account_metas(None);

    airdrop(&mut context, &st, 10_000_000_000).await.unwrap();

    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::Cancel {
            _buyer_price: 1,
            _token_size: 1,
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
