#![cfg(feature = "test-bpf")]
#![allow(dead_code)]

use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use anchor_client::solana_client::rpc_client::RpcClient;
use solana_gateway::{
    instruction::{self, NetworkFeature},
    state::{
        get_gatekeeper_address_with_seed, get_gateway_token_address_with_seed, GatewayTokenState,
    },
};
use solana_program::pubkey;
use solana_program_test::*;
use solana_sdk::{
    clock::UnixTimestamp,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use mpl_candy_machine::{
    CandyMachineData, GatekeeperConfig as GKConfig, WhitelistMintMode::BurnEveryTime,
};
use utils::{custom_config, GatekeeperInfo};

use crate::{
    core::helpers::{airdrop, assert_account_empty, get_balance},
    utils::{
        auto_config, candy_machine_program_test,
        helpers::{sol, test_start},
        CandyManager, GatekeeperConfig, WhitelistConfig,
    },
};

const GATEWAY_ACCOUNT_PUBKEY: Pubkey = pubkey!("gatem74V238djXdzWnJf94Wo1DcnuGkfijbf3AuBhfs");

mod core;
mod utils;

#[tokio::test]
async fn init_default_success() {
    test_start("Init Default Success");
    let mut context = candy_machine_program_test().start_with_context().await;
    let context = &mut context;

    let mut candy_manager = CandyManager::init(
        context,
        Some(true),
        true,
        None,
        Some(WhitelistConfig::new(BurnEveryTime, false, Some(1))),
        None,
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
    candy_manager.set_collection(context).await.unwrap();

    let failed = candy_manager.mint_and_assert_bot_tax(context).await;
    if failed.is_err() {
        println!("Had an error when it potentially should have been bot tax!");
    }
    let candy_data = CandyMachineData {
        go_live_date: Some(0),
        price: 1,
        ..candy_data
    };
    candy_manager
        .update(context, None, candy_data)
        .await
        .unwrap();
    candy_manager
        .mint_and_assert_successful(context, Some(1), true)
        .await
        .unwrap();
    let pre_balance = get_balance(context, &candy_manager.authority.pubkey()).await;
    candy_manager.withdraw(context).await.unwrap();
    let post_balance = get_balance(context, &candy_manager.authority.pubkey()).await;
    assert_account_empty(context, &candy_manager.candy_machine.pubkey()).await;
    assert_account_empty(context, &candy_manager.collection_info.pda).await;
    assert!(post_balance > pre_balance);
}
