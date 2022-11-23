#![cfg(feature = "test-bpf")]
#![allow(dead_code)]

use crate::utils::{auto_config, candy_machine_program_test, helpers::test_start, CandyManager};
use solana_program_test::*;
use solana_sdk::signature::{Keypair, Signer};

mod core;
mod utils;

#[tokio::test]
async fn update_auth_with_collection_fails() {
    test_start("Update Authority With Collection Fails");
    let mut context = candy_machine_program_test().start_with_context().await;
    let context = &mut context;

    let mut candy_manager = CandyManager::init(context, Some(true), false, None, None, None).await;

    let candy_data = auto_config(&candy_manager, Some(0), true, true, None, None);
    candy_manager
        .create(context, candy_data.clone())
        .await
        .unwrap();
    candy_manager.fill_config_lines(context).await.unwrap();
    candy_manager.set_collection(context).await.unwrap();

    let new_authority = Keypair::new();
    candy_manager
        .update_authority(context, new_authority.pubkey())
        .await
        .unwrap_err();

    candy_manager.remove_collection(context).await.unwrap();

    candy_manager
        .update_authority(context, new_authority.pubkey())
        .await
        .unwrap();
    let candy_machine_account = candy_manager.get_candy(context).await;
    assert_eq!(
        new_authority.pubkey(),
        candy_machine_account.authority,
        "Authority wasn't updated correctly!"
    );
}
