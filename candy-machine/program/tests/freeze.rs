#![cfg(feature = "test-bpf")]

use solana_program::program_option::COption;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};
use spl_token::state::AccountState;

use mpl_candy_machine::{CandyMachineData, WhitelistMintMode::BurnEveryTime};

use crate::{
    core::helpers::{airdrop, assert_account_empty, clone_keypair, get_token_account},
    utils::{auto_config, candy_machine_program_test, helpers::sol, CandyManager, WhitelistConfig},
};

mod core;
mod utils;

#[tokio::test]
async fn test_freeze() {
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
    let token_account = get_token_account(context, &new_nft.token_account)
        .await
        .unwrap();
    println!("Token account: {:#?}", token_account);
    assert_eq!(
        token_account.state,
        AccountState::Frozen,
        "Token account state is not correct"
    );
    assert_eq!(
        token_account.delegate,
        COption::Some(candy_manager.freeze_info.pda),
        "Token account delegate is not correct"
    );
    assert_eq!(
        token_account.delegated_amount, 1,
        "Delegated amount is not correct"
    );
    candy_manager
        .thaw_nft(
            context,
            new_nft.clone(),
            &new_nft.owner,
            &clone_keypair(&candy_manager.authority),
        )
        .await
        .unwrap_err();
    let token_account = get_token_account(context, &new_nft.token_account)
        .await
        .unwrap();
    assert_eq!(
        token_account.state,
        AccountState::Frozen,
        "Token account state is not correct"
    );
    assert_eq!(
        token_account.delegate,
        COption::Some(candy_manager.freeze_info.pda),
        "Token account delegate is not correct"
    );
    assert_eq!(
        token_account.delegated_amount, 1,
        "Delegated amount is not correct"
    );
    println!("Token account: {:#?}", token_account);
    candy_manager.remove_freeze(context).await.unwrap();
    let freeze_pda = candy_manager.get_freeze_pda(context).await;
    println!("Freeze PDA: {:#?}", freeze_pda);
    assert!(freeze_pda.allow_thaw, "Allow thaw is not true!");
    candy_manager
        .thaw_nft(
            context,
            new_nft.clone(),
            &new_nft.owner,
            &clone_keypair(&candy_manager.authority),
        )
        .await
        .unwrap();
    let token_account = get_token_account(context, &new_nft.token_account)
        .await
        .unwrap();
    assert_eq!(
        token_account.state,
        AccountState::Initialized,
        "Token account state is not correct"
    );
    assert_eq!(
        token_account.delegate,
        COption::Some(candy_manager.freeze_info.pda),
        "Token account delegate is not correct"
    );
    assert_eq!(
        token_account.delegated_amount, 1,
        "Delegated amount is not correct"
    );
    println!("Token account: {:#?}", token_account);
    candy_manager
        .thaw_nft(context, new_nft.clone(), &new_nft.owner, &new_nft.owner)
        .await
        .unwrap();
    let token_account = get_token_account(context, &new_nft.token_account)
        .await
        .unwrap();
    println!("Token account: {:#?}", token_account);
    assert_eq!(
        token_account.state,
        AccountState::Initialized,
        "Token account state is not Initialized"
    );
    assert_eq!(
        token_account.delegate,
        COption::None,
        "Token account delegate is not none"
    );
    assert_eq!(
        token_account.delegated_amount, 0,
        "Delegated amount is not correct"
    );
}
