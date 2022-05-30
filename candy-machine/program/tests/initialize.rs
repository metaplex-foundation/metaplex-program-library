#![cfg(feature = "test-bpf")]

use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

use mpl_candy_machine::{CandyMachineData, WhitelistMintMode::BurnEveryTime};

use crate::{
    core::helpers::{airdrop, assert_account_empty, clone_keypair, get_token_account},
    utils::{auto_config, candy_machine_program_test, helpers::sol, CandyManager, WhitelistConfig},
};

mod core;
mod utils;

#[tokio::test]
async fn init_everything() {
    let mut context = candy_machine_program_test().start_with_context().await;
    let context = &mut context;

    let mut candy_manager = CandyManager::init(
        context,
        true,
        true,
        true,
        Some(WhitelistConfig::new(BurnEveryTime, false, Some(1))),
    )
    .await;

    airdrop(context, &candy_manager.minter.pubkey(), sol(2.0))
        .await
        .unwrap();
    let candy_data = auto_config(&candy_manager, None, true, true, None, None);
    candy_manager
        .create(context, candy_data.clone())
        .await
        .unwrap();
    candy_manager.fill_config_lines(context).await.unwrap();
    assert_account_empty(context, &candy_manager.collection_info.pda).await;
    candy_manager.set_collection(context).await.unwrap();
    let collection_pda_account = candy_manager.get_collection_pda(context).await;
    println!("Collection PDA: {:#?}", collection_pda_account);
    assert_account_empty(context, &candy_manager.freeze_info.pda).await;
    candy_manager.set_freeze(context).await.unwrap();
    let freeze_pda_account = candy_manager.get_freeze_pda(context).await;
    println!("Freeze PDA: {:#?}", freeze_pda_account);
    candy_manager.remove_freeze(context).await.unwrap();
    assert_account_empty(context, &candy_manager.freeze_info.pda).await;
    candy_manager.remove_collection(context).await.unwrap();
    assert_account_empty(context, &candy_manager.collection_info.pda).await;
}
