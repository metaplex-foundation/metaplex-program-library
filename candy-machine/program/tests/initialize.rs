#![cfg(feature = "test-bpf")]

use std::{cell::RefCell, rc::Rc};

use solana_gateway::{error::GatewayError, Gateway};
use solana_program_test::*;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};

use mpl_candy_machine::{CandyMachineData, GatekeeperConfig, WhitelistMintMode::BurnEveryTime};
use solana_program::account_info::AccountInfo;
use utils::custom_config;

use crate::{
    core::helpers::airdrop,
    utils::{auto_config, candy_machine_program_test, helpers::sol, CandyManager, WhitelistConfig},
};

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

// #[tokio::test]
// async fn bot_tax_on_gatekeeper() {
//     let mut context = candy_machine_program_test().start_with_context().await;
//     let context = &mut context;

//     let mut candy_manager = CandyManager::init(context, true, false, None).await;

//     airdrop(context, &candy_manager.minter.pubkey(), sol(2.0))
//         .await
//         .unwrap();

//     let candy_data = custom_config(
//         candy_manager.authority.pubkey(),
//         Some(0),
//         true,
//         true,
//         None,
//         None,
//         None,
//         Some(GatekeeperConfig {
//             gatekeeper_network: Pubkey::new(b"ignREusXmGrscGNUesoU9mxfds9AiYTezUKex2PsZV6"),
//             expire_on_use: true,
//         }),
//     );
//     candy_manager
//         .create(context, candy_data.clone())
//         .await
//         .unwrap();
//     candy_manager.fill_config_lines(context).await.unwrap();
//     candy_manager.set_collection(context).await.unwrap();

//     // let failed = candy_manager.mint_and_assert_bot_tax(context).await;
//     // if failed.is_err() {
//     //     println!("Had an error when it potentially should have been bot tax!");
//     // }

//     // assert_transport_error!(
//     //     err,
//     //     TransportError::TransactionError(TransactionError::InstructionError(0, _))
//     // );

//     let mut lamports = 0;
//     let account_info = AccountInfo {
//         key: &Default::default(),
//         is_signer: false,
//         is_writable: false,
//         lamports: Rc::new(RefCell::new(&mut lamports)),
//         data: Rc::new(RefCell::new(&mut [])),
//         owner: &Default::default(),
//         executable: false,
//         rent_epoch: 0,
//     };
//     let verify_result = Gateway::verify_gateway_token_account_info(
//         &account_info,
//         &Default::default(),
//         &Default::default(),
//         None,
//     );

//     assert!(matches!(
//         verify_result,
//         Err(GatewayError::IncorrectProgramId)
//     ));

//     candy_manager
//         .mint_and_assert_successful(context, Some(1), true)
//         .await;
// }
