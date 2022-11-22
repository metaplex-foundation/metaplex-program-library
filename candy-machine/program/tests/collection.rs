#![cfg(feature = "test-bpf")]
#![allow(dead_code)]

use mpl_candy_machine::CandyError;
use mpl_token_metadata::state::CollectionDetails;
use solana_program::instruction::InstructionError;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::TransactionError,
};

use crate::{
    core::helpers::airdrop,
    utils::{
        auto_config, candy_machine_program_test,
        helpers::{sol, test_start},
        CandyManager,
    },
};

mod core;
mod utils;

#[tokio::test]
async fn mint_sized_collections() {
    test_start("Mint Sized Collections");
    let mut context = candy_machine_program_test().start_with_context().await;
    let context = &mut context;

    let mut candy_manager = CandyManager::init(context, Some(true), false, None, None, None).await;

    airdrop(context, &candy_manager.minter.pubkey(), sol(5.0))
        .await
        .unwrap();
    let candy_data = auto_config(&candy_manager, Some(0), true, true, None, None);
    candy_manager
        .create(context, candy_data.clone())
        .await
        .unwrap();
    candy_manager.fill_config_lines(context).await.unwrap();
    candy_manager.set_collection(context).await.unwrap();

    candy_manager
        .mint_and_assert_successful(context, Some(sol(1.0)), true)
        .await
        .unwrap();

    let collection_metadata = candy_manager.collection_info.get_metadata(context).await;

    assert_eq!(
        collection_metadata.collection_details,
        Some(CollectionDetails::V1 { size: 1 }),
        "Sized collection not set correctly."
    );

    candy_manager
        .mint_and_assert_successful(context, Some(sol(1.0)), true)
        .await
        .unwrap();

    let collection_metadata = candy_manager.collection_info.get_metadata(context).await;

    assert_eq!(
        collection_metadata.collection_details,
        Some(CollectionDetails::V1 { size: 2 }),
        "Sized collection not set correctly."
    );
}

#[tokio::test]
async fn throws_sized_writable_error() {
    test_start("Sized Collections Throws Writable Error");
    let mut context = candy_machine_program_test().start_with_context().await;
    let context = &mut context;

    let mut candy_manager = CandyManager::init(context, Some(true), false, None, None, None).await;

    airdrop(context, &candy_manager.minter.pubkey(), sol(5.0))
        .await
        .unwrap();
    let candy_data = auto_config(&candy_manager, Some(0), true, true, None, None);
    candy_manager
        .create(context, candy_data.clone())
        .await
        .unwrap();
    candy_manager.fill_config_lines(context).await.unwrap();
    candy_manager.set_collection(context).await.unwrap();

    // fake remove the sized collection to send the account in as non writable!
    candy_manager.collection_info.sized = false;

    let error_num = match candy_manager.mint_nft(context).await.unwrap_err().unwrap() {
        TransactionError::InstructionError(_, InstructionError::Custom(err_num)) => err_num,
        _ => 0,
    };
    let candy_error_num: u32 = CandyError::SizedCollectionMetadataMustBeMutable.into();
    assert_eq!(error_num, candy_error_num);
}
