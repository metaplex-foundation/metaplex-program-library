#![cfg(feature = "test-bpf")]
#![allow(dead_code)]

use solana_program::clock::Clock;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

use mpl_candy_machine::{
    constants::{FREEZE_FEATURE_INDEX, FREEZE_FEE, FREEZE_LOCK_FEATURE_INDEX, MAX_FREEZE_TIME},
    is_feature_active, CandyMachineData, FreezePDA,
    WhitelistMintMode::BurnEveryTime,
};

use crate::{
    core::helpers::{
        airdrop, assert_account_empty, clone_keypair, get_balance, get_token_balance,
        new_funded_keypair,
    },
    utils::{
        auto_config, candy_machine_program_test,
        helpers::{sol, test_start},
        CandyManager, FreezeConfig, WhitelistConfig,
    },
};

pub mod core;
pub mod utils;

#[tokio::test]
async fn freeze_flow_with_spl_token() {
    test_start("Test Freeze With SPL Token");
    let mut context = candy_machine_program_test().start_with_context().await;
    let context = &mut context;

    let freeze_time = 60 * 60;
    let mut candy_manager = CandyManager::init(
        context,
        Some(false),
        true,
        Some(FreezeConfig::new(true, freeze_time)),
        Some(WhitelistConfig::new(BurnEveryTime, false, Some(1))),
        None,
    )
    .await;
    let balance = get_token_balance(context, &candy_manager.token_info.auth_account).await;
    let balance2 = get_token_balance(context, &candy_manager.token_info.minter_account).await;
    println!("Auth: {}, Minter: {}", balance, balance2);

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

    assert_account_empty(context, &candy_manager.freeze_info.pda).await;
    candy_manager.set_freeze(context).await.unwrap();

    let mut expected_freeze_pda = FreezePDA {
        candy_machine: candy_manager.candy_machine.pubkey(),
        freeze_fee: FREEZE_FEE,
        freeze_time,
        frozen_count: 0,
        allow_thaw: false,
        mint_start: None,
    };

    candy_manager
        .assert_freeze_set(context, &expected_freeze_pda)
        .await;

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

    let new_nft = candy_manager
        .mint_and_assert_successful(context, Some(1), true)
        .await
        .unwrap();
    let mint_start = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp;
    expected_freeze_pda.mint_start = Some(mint_start);
    expected_freeze_pda.frozen_count += 1;

    candy_manager.assert_frozen(context, &new_nft).await;
    candy_manager
        .assert_freeze_set(context, &expected_freeze_pda)
        .await;

    candy_manager
        .thaw_nft(context, &new_nft, &clone_keypair(&candy_manager.authority))
        .await
        .unwrap_err();
    candy_manager.assert_frozen(context, &new_nft).await;

    candy_manager.remove_freeze(context).await.unwrap();
    let freeze_pda = candy_manager.get_freeze_pda(context).await;
    assert!(freeze_pda.allow_thaw, "Allow thaw is not true!");

    candy_manager
        .thaw_nft(context, &new_nft, &clone_keypair(&candy_manager.authority))
        .await
        .unwrap();

    candy_manager.assert_thawed(context, &new_nft, false).await;

    candy_manager
        .thaw_nft(context, &new_nft, &new_nft.owner)
        .await
        .unwrap();
    candy_manager.assert_thawed(context, &new_nft, true).await;

    let pre_balance = get_token_balance(context, &candy_manager.token_info.auth_account).await;
    candy_manager.unlock_funds(context).await.unwrap();
    let post_balance = get_token_balance(context, &candy_manager.token_info.auth_account).await;
    assert!(post_balance - pre_balance >= 1);
}

#[tokio::test]
async fn freeze_update() {
    test_start("Test Freeze Update");
    let mut context = candy_machine_program_test().start_with_context().await;
    let context = &mut context;
    let freeze_time = 60 * 60;
    let mut candy_manager = CandyManager::init(
        context,
        None,
        false,
        Some(FreezeConfig::new(true, freeze_time)),
        None,
        None,
    )
    .await;

    let random_key = new_funded_keypair(context, sol(1.0)).await;
    airdrop(context, &candy_manager.minter.pubkey(), sol(20.0))
        .await
        .unwrap();

    let candy_data = auto_config(&candy_manager, Some(0), true, true, None, None);
    candy_manager
        .create(context, candy_data.clone())
        .await
        .unwrap();
    candy_manager.fill_config_lines(context).await.unwrap();

    assert_account_empty(context, &candy_manager.freeze_info.pda).await;
    candy_manager.set_freeze(context).await.unwrap();

    let mut expected_freeze_pda = FreezePDA {
        candy_machine: candy_manager.candy_machine.pubkey(),
        freeze_fee: FREEZE_FEE,
        freeze_time,
        frozen_count: 0,
        allow_thaw: false,
        mint_start: None,
    };
    candy_manager
        .assert_freeze_set(context, &expected_freeze_pda)
        .await;

    candy_manager.remove_freeze(context).await.unwrap();

    let candy_machine_account = candy_manager.get_candy(context).await;
    assert_account_empty(context, &candy_manager.freeze_info.pda).await;
    assert!(!is_feature_active(
        &candy_machine_account.data.uuid,
        FREEZE_FEATURE_INDEX
    ));
    assert!(!is_feature_active(
        &candy_machine_account.data.uuid,
        FREEZE_LOCK_FEATURE_INDEX
    ));

    candy_manager.set_freeze(context).await.unwrap();
    candy_manager
        .assert_freeze_set(context, &expected_freeze_pda)
        .await;

    let new_nft = candy_manager
        .mint_and_assert_successful(context, Some(sol(1.0)), true)
        .await
        .unwrap();
    candy_manager.assert_frozen(context, &new_nft).await;

    let mint_start = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp;
    expected_freeze_pda.mint_start = Some(mint_start);
    expected_freeze_pda.frozen_count += 1;

    candy_manager
        .assert_freeze_set(context, &expected_freeze_pda)
        .await;

    candy_manager
        .thaw_nft(context, &new_nft, &clone_keypair(&candy_manager.authority))
        .await
        .unwrap_err();

    candy_manager.remove_freeze(context).await.unwrap();

    expected_freeze_pda.allow_thaw = true;
    let freeze_pda = candy_manager.get_freeze_pda(context).await;
    assert_eq!(freeze_pda, expected_freeze_pda);
    let uuid = candy_manager.get_candy(context).await.data.uuid;
    assert!(!is_feature_active(&uuid, FREEZE_FEATURE_INDEX));
    assert!(is_feature_active(&uuid, FREEZE_LOCK_FEATURE_INDEX));

    candy_manager
        .thaw_nft(context, &new_nft, &random_key)
        .await
        .unwrap();
    candy_manager.assert_thawed(context, &new_nft, false).await;
    let freeze_pda = candy_manager.get_freeze_pda(context).await;
    expected_freeze_pda.frozen_count -= 1;
    assert_eq!(freeze_pda, expected_freeze_pda);

    let new_nft_2 = candy_manager
        .mint_and_assert_successful(context, Some(sol(1.0)), true)
        .await
        .unwrap();
    candy_manager.assert_thawed(context, &new_nft_2, true).await;

    let freeze_pda_before = candy_manager.get_freeze_pda(context).await;
    candy_manager
        .thaw_nft(
            context,
            &new_nft_2,
            &clone_keypair(&candy_manager.authority),
        )
        .await
        .unwrap();
    let freeze_pda_after = candy_manager.get_freeze_pda(context).await;
    assert_eq!(freeze_pda_before, freeze_pda_after);
}

#[tokio::test]
async fn thaw_after_freeze_time() {
    test_start("Thaw After Freeze Time");
    let mut context = candy_machine_program_test().start_with_context().await;
    let context = &mut context;
    let freeze_time = 30; //30 seconds
    let mut candy_manager = CandyManager::init(
        context,
        None,
        false,
        Some(FreezeConfig::new(true, freeze_time)),
        None,
        None,
    )
    .await;

    airdrop(context, &candy_manager.minter.pubkey(), sol(20.0))
        .await
        .unwrap();

    let candy_data = auto_config(&candy_manager, Some(0), true, true, None, None);
    candy_manager
        .create(context, candy_data.clone())
        .await
        .unwrap();
    candy_manager.fill_config_lines(context).await.unwrap();

    assert_account_empty(context, &candy_manager.freeze_info.pda).await;
    candy_manager.set_freeze(context).await.unwrap();

    let expected_freeze_pda = FreezePDA {
        candy_machine: candy_manager.candy_machine.pubkey(),
        freeze_fee: FREEZE_FEE,
        freeze_time,
        frozen_count: 0,
        allow_thaw: false,
        mint_start: None,
    };
    candy_manager
        .assert_freeze_set(context, &expected_freeze_pda)
        .await;
    let new_nft = candy_manager
        .mint_and_assert_successful(context, Some(sol(1.0)), true)
        .await
        .unwrap();
    let current_slot = context.banks_client.get_root_slot().await.unwrap();

    //test thaw fail
    candy_manager
        .thaw_nft(context, &new_nft, &new_nft.authority)
        .await
        .unwrap_err();

    context.warp_to_slot(current_slot + 20000).unwrap();
    candy_manager
        .thaw_nft(context, &new_nft, &new_nft.authority)
        .await
        .unwrap();
    let thaw_time = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp;

    candy_manager.assert_thawed(context, &new_nft, false).await;

    let mint_time = candy_manager
        .get_freeze_pda(context)
        .await
        .mint_start
        .unwrap();
    assert!(
        thaw_time - mint_time >= freeze_time,
        "This shouldn't happen. Something must have went wrong."
    );

    // now that freeze time has passed, new mints shouldn't be frozen
    let new_nft = candy_manager
        .mint_and_assert_successful(context, Some(sol(1.0)), true)
        .await
        .unwrap();
    candy_manager.assert_thawed(context, &new_nft, true).await;
}

#[tokio::test]
async fn unlock_funds() {
    test_start("Unlock Funds");
    let mut context = candy_machine_program_test().start_with_context().await;
    let context = &mut context;
    let freeze_time = 30; //30 seconds
    let mut candy_manager = CandyManager::init(
        context,
        None,
        false,
        Some(FreezeConfig::new(true, freeze_time)),
        None,
        None,
    )
    .await;

    airdrop(context, &candy_manager.minter.pubkey(), sol(20.0))
        .await
        .unwrap();

    let candy_data = auto_config(&candy_manager, Some(0), true, true, None, None);
    candy_manager
        .create(context, candy_data.clone())
        .await
        .unwrap();
    candy_manager.fill_config_lines(context).await.unwrap();

    assert_account_empty(context, &candy_manager.freeze_info.pda).await;
    candy_manager.set_freeze(context).await.unwrap();

    let expected_freeze_pda = FreezePDA {
        candy_machine: candy_manager.candy_machine.pubkey(),
        freeze_fee: FREEZE_FEE,
        freeze_time,
        frozen_count: 0,
        allow_thaw: false,
        mint_start: None,
    };
    candy_manager
        .assert_freeze_set(context, &expected_freeze_pda)
        .await;

    let new_nft = candy_manager
        .mint_and_assert_successful(context, Some(sol(1.0)), true)
        .await
        .unwrap();

    candy_manager.remove_freeze(context).await.unwrap();
    // shouldn't work because one nft is still frozen
    candy_manager.unlock_funds(context).await.unwrap_err();
    candy_manager
        .thaw_nft(context, &new_nft, &new_nft.owner)
        .await
        .unwrap();
    candy_manager.assert_thawed(context, &new_nft, true).await;
    let pre_balance = get_balance(context, &candy_manager.authority.pubkey()).await;
    candy_manager.unlock_funds(context).await.unwrap();
    let post_balance = get_balance(context, &candy_manager.authority.pubkey()).await;
    assert!(post_balance - pre_balance >= sol(1.0));
}

#[tokio::test]
async fn withdraw_funds() {
    test_start("Withdraw Funds");
    let mut context = candy_machine_program_test().start_with_context().await;
    let context = &mut context;
    let freeze_time = 30; //30 seconds
    let mut candy_manager = CandyManager::init(
        context,
        None,
        false,
        Some(FreezeConfig::new(true, freeze_time)),
        None,
        None,
    )
    .await;

    airdrop(context, &candy_manager.minter.pubkey(), sol(20.0))
        .await
        .unwrap();

    let candy_data = auto_config(&candy_manager, Some(0), true, true, None, None);
    candy_manager
        .create(context, candy_data.clone())
        .await
        .unwrap();
    candy_manager.fill_config_lines(context).await.unwrap();

    assert_account_empty(context, &candy_manager.freeze_info.pda).await;
    candy_manager.set_freeze(context).await.unwrap();

    let expected_freeze_pda = FreezePDA {
        candy_machine: candy_manager.candy_machine.pubkey(),
        freeze_fee: FREEZE_FEE,
        freeze_time,
        frozen_count: 0,
        allow_thaw: false,
        mint_start: None,
    };
    candy_manager
        .assert_freeze_set(context, &expected_freeze_pda)
        .await;

    let new_nft = candy_manager
        .mint_and_assert_successful(context, Some(sol(1.0)), true)
        .await
        .unwrap();

    candy_manager.remove_freeze(context).await.unwrap();
    // shouldn't work because one nft is still frozen
    candy_manager.unlock_funds(context).await.unwrap_err();
    candy_manager
        .thaw_nft(context, &new_nft, &new_nft.owner)
        .await
        .unwrap();
    candy_manager.assert_thawed(context, &new_nft, true).await;
    // candy_manager.
    candy_manager.withdraw(context).await.unwrap_err();
    let pre_balance = get_balance(context, &candy_manager.authority.pubkey()).await;
    candy_manager.unlock_funds(context).await.unwrap();
    let post_balance = get_balance(context, &candy_manager.authority.pubkey()).await;
    assert!(post_balance - pre_balance >= sol(1.0));
    candy_manager.withdraw(context).await.unwrap();
    let post_post_balance = get_balance(context, &candy_manager.authority.pubkey()).await;
    assert!(post_post_balance > post_balance);
    assert_account_empty(context, &candy_manager.candy_machine.pubkey()).await;
}

#[tokio::test]
async fn mint_out_unfreeze() {
    test_start("Mint Out Unfreeze");
    let mut context = candy_machine_program_test().start_with_context().await;
    let context = &mut context;
    let freeze_time = MAX_FREEZE_TIME;
    let mut candy_manager = CandyManager::init(
        context,
        None,
        false,
        Some(FreezeConfig::new(true, freeze_time)),
        None,
        None,
    )
    .await;
    let random_key = new_funded_keypair(context, sol(1.0)).await;
    airdrop(context, &candy_manager.minter.pubkey(), sol(6.0))
        .await
        .unwrap();

    let mut candy_data = auto_config(&candy_manager, Some(0), true, true, None, None);
    candy_data.items_available = 2;
    candy_manager
        .create(context, candy_data.clone())
        .await
        .unwrap();
    candy_manager.fill_config_lines(context).await.unwrap();

    assert_account_empty(context, &candy_manager.freeze_info.pda).await;
    candy_manager.set_freeze(context).await.unwrap();

    let expected_freeze_pda = FreezePDA {
        candy_machine: candy_manager.candy_machine.pubkey(),
        freeze_fee: FREEZE_FEE,
        freeze_time,
        frozen_count: 0,
        allow_thaw: false,
        mint_start: None,
    };

    candy_manager
        .assert_freeze_set(context, &expected_freeze_pda)
        .await;

    let nft1 = candy_manager
        .mint_and_assert_successful(context, Some(sol(1.0)), true)
        .await
        .unwrap();

    candy_manager.assert_frozen(context, &nft1).await;

    // should fail
    candy_manager
        .thaw_nft(context, &nft1, &random_key)
        .await
        .unwrap_err();

    let nft2 = candy_manager
        .mint_and_assert_successful(context, Some(sol(1.0)), true)
        .await
        .unwrap();

    candy_manager.assert_frozen(context, &nft2).await;

    // should succeed
    candy_manager
        .thaw_nft(context, &nft1, &random_key)
        .await
        .unwrap();

    // This should fail because nft2 is still frozen
    candy_manager.unlock_funds(context).await.unwrap_err();

    candy_manager
        .thaw_nft(context, &nft2, &random_key)
        .await
        .unwrap();

    let pre_balance = get_balance(context, &candy_manager.authority.pubkey()).await;
    candy_manager.unlock_funds(context).await.unwrap();
    let post_balance = get_balance(context, &candy_manager.authority.pubkey()).await;
    assert!(post_balance - pre_balance >= sol(2.0));
}
