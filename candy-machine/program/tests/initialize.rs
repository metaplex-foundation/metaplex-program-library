#![cfg(feature = "test-bpf")]

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

use crate::{
    core::helpers::airdrop,
    utils::{
        auto_config, candy_machine_program_test, helpers::sol, CandyManager, GatekeeperConfig,
        WhitelistConfig,
    },
};
use mpl_candy_machine::{
    CandyMachineData, GatekeeperConfig as GKConfig, WhitelistMintMode::BurnEveryTime,
};
use utils::{custom_config, GatekeeperInfo};

const GATEWAY_ACCOUNT_PUBKEY: Pubkey = pubkey!("gatem74V238djXdzWnJf94Wo1DcnuGkfijbf3AuBhfs");

mod core;
mod utils;

#[tokio::test]
async fn init_default_success() {
    let mut context = candy_machine_program_test().start_with_context().await;
    let context = &mut context;

    let mut candy_manager = CandyManager::init(
        context,
        true,
        true,
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
        .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bot_tax_on_gatekeeper_expire_token() {
    let mut context = candy_machine_program_test().start_with_context().await;
    let context = &mut context;

    let gatekeeper_network = Keypair::new();
    let gatekeeper_authority = Keypair::new();

    let client = RpcClient::new("https://metaplex.devnet.rpcpool.com".to_string());

    let gateway_account_pubkey = GATEWAY_ACCOUNT_PUBKEY;
    let gateway_executable_pubkey =
        Pubkey::from_str("D5iXG4Z4hajVFAs8UbmBwdfe7PFqvoT4LNVvt1nKU5bx").unwrap();
    let gateway_account = client.get_account(&gateway_account_pubkey).unwrap();
    let gateway_executable_account = client.get_account(&gateway_executable_pubkey).unwrap();
    context.set_account(&gateway_account_pubkey, &gateway_account.into());
    context.set_account(
        &gateway_executable_pubkey,
        &gateway_executable_account.into(),
    );

    let mut candy_manager = CandyManager::init(
        context,
        false,
        false,
        None,
        Some(GatekeeperInfo {
            set: true,
            network_expire_feature: None,
            gateway_app: GATEWAY_ACCOUNT_PUBKEY,
            gateway_token_info: gatekeeper_network.pubkey(),
            gatekeeper_config: GatekeeperConfig {
                gatekeeper_network: gatekeeper_network.pubkey(),
                expire_on_use: true,
            },
        }),
    )
    .await;

    airdrop(context, &candy_manager.minter.pubkey(), sol(2.0))
        .await
        .unwrap();

    let candy_data = custom_config(
        candy_manager.authority.pubkey(),
        Some(0),
        true,
        true,
        None,
        None,
        None,
        Some(GKConfig {
            gatekeeper_network: gatekeeper_network.pubkey(),
            expire_on_use: true,
        }),
    );

    candy_manager
        .create(context, candy_data.clone())
        .await
        .unwrap();
    candy_manager.fill_config_lines(context).await.unwrap();

    let block_hash = context.banks_client.get_latest_blockhash().await.unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[instruction::add_gatekeeper(
            &candy_manager.minter.pubkey(),
            &gatekeeper_authority.pubkey(),
            &gatekeeper_network.pubkey(),
        )],
        Some(&candy_manager.minter.pubkey()),
        &[&candy_manager.minter, &gatekeeper_network],
        block_hash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let (gatekeeper_account, _) = get_gatekeeper_address_with_seed(
        &gatekeeper_authority.pubkey(),
        &gatekeeper_network.pubkey(),
    );

    let start = SystemTime::now();
    let now = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    // creating with an already expired token to fail the mint
    let block_hash = context.banks_client.get_latest_blockhash().await.unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[instruction::issue_vanilla(
            &context.payer.pubkey(),
            &candy_manager.minter.pubkey(),
            &gatekeeper_account,
            &gatekeeper_authority.pubkey(),
            &gatekeeper_network.pubkey(),
            None,
            Some(now.as_secs() as UnixTimestamp - 10),
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &gatekeeper_authority],
        block_hash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let block_hash = context.banks_client.get_latest_blockhash().await.unwrap();
    context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction::add_feature_to_network(
                context.payer.pubkey(),
                gatekeeper_network.pubkey(),
                NetworkFeature::UserTokenExpiry,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, &gatekeeper_network],
            block_hash,
        ))
        .await
        .unwrap();

    candy_manager
        .mint_and_assert_bot_tax(context)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bot_tax_on_gatekeeper() {
    let mut context = candy_machine_program_test().start_with_context().await;
    let context = &mut context;

    let gatekeeper_network = Keypair::new();
    let gatekeeper_authority = Keypair::new();

    let client = RpcClient::new("https://metaplex.devnet.rpcpool.com".to_string());

    let gateway_account_pubkey = GATEWAY_ACCOUNT_PUBKEY;
    let gateway_executable_pubkey =
        Pubkey::from_str("D5iXG4Z4hajVFAs8UbmBwdfe7PFqvoT4LNVvt1nKU5bx").unwrap();
    let gateway_account = client.get_account(&gateway_account_pubkey).unwrap();
    let gateway_executable_account = client.get_account(&gateway_executable_pubkey).unwrap();
    context.set_account(&gateway_account_pubkey, &gateway_account.into());
    context.set_account(
        &gateway_executable_pubkey,
        &gateway_executable_account.into(),
    );

    let mut candy_manager = CandyManager::init(
        context,
        false,
        false,
        None,
        Some(GatekeeperInfo {
            set: true,
            network_expire_feature: None,
            gateway_app: GATEWAY_ACCOUNT_PUBKEY,
            gateway_token_info: gatekeeper_network.pubkey(),
            gatekeeper_config: GatekeeperConfig {
                gatekeeper_network: gatekeeper_network.pubkey(),
                expire_on_use: false,
            },
        }),
    )
    .await;

    airdrop(context, &candy_manager.minter.pubkey(), sol(2.0))
        .await
        .unwrap();

    let candy_data = custom_config(
        candy_manager.authority.pubkey(),
        Some(0),
        true,
        true,
        None,
        None,
        None,
        Some(GKConfig {
            gatekeeper_network: gatekeeper_network.pubkey(),
            expire_on_use: false,
        }),
    );

    candy_manager
        .create(context, candy_data.clone())
        .await
        .unwrap();
    candy_manager.fill_config_lines(context).await.unwrap();

    let block_hash = context.banks_client.get_latest_blockhash().await.unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[instruction::add_gatekeeper(
            &candy_manager.minter.pubkey(),
            &gatekeeper_authority.pubkey(),
            &gatekeeper_network.pubkey(),
        )],
        Some(&candy_manager.minter.pubkey()),
        &[&candy_manager.minter, &gatekeeper_network],
        block_hash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let (gatekeeper_account, _) = get_gatekeeper_address_with_seed(
        &gatekeeper_authority.pubkey(),
        &gatekeeper_network.pubkey(),
    );

    let block_hash = context.banks_client.get_latest_blockhash().await.unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[instruction::issue_vanilla(
            &context.payer.pubkey(),
            &candy_manager.minter.pubkey(),
            &gatekeeper_account,
            &gatekeeper_authority.pubkey(),
            &gatekeeper_network.pubkey(),
            None,
            None,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &gatekeeper_authority],
        block_hash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let (gateway_account, _) = get_gateway_token_address_with_seed(
        &candy_manager.minter.pubkey(),
        &None,
        &gatekeeper_network.pubkey(),
    );

    let block_hash = context.banks_client.get_latest_blockhash().await.unwrap();
    // revoking the token so verification fails inside of gateway triggering bot tax
    let transaction = Transaction::new_signed_with_payer(
        &[instruction::set_state(
            &gateway_account,
            &gatekeeper_authority.pubkey(),
            &gatekeeper_account,
            GatewayTokenState::Revoked,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &gatekeeper_authority],
        block_hash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    candy_manager
        .mint_and_assert_bot_tax(context)
        .await
        .unwrap();
}
