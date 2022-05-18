#![cfg(feature = "test-bpf")]

use anchor_lang::AccountDeserialize;
use mpl_token_metadata::state::Metadata;
use solana_program::native_token::{Sol, LAMPORTS_PER_SOL};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

use mpl_candy_machine::{CandyMachine, CandyMachineData};
use utils::helper_transactions;

use crate::core::{clone_keypair, get_account, metadata};
use crate::utils::candy_manager::CandyManager;
use crate::utils::sol;
use crate::{core::airdrop, utils::add_all_config_lines};

mod core;
mod utils;

#[tokio::test]
async fn init_default_success() {
    let mut context = helper_transactions::candy_machine_program_test()
        .start_with_context()
        .await;

    let mut candy_manager = CandyManager::new();

    airdrop(&mut context, &candy_manager.authority.pubkey(), sol(10f64))
        .await
        .unwrap();
    let start_uuid = "123456".to_string();
    let candy_data = CandyMachineData {
        uuid: start_uuid.clone(),
        items_available: 500,
        price: sol(0.1),
        ..CandyMachineData::default()
    };
    candy_manager
        .create(&mut context, candy_data)
        .await
        .unwrap();

    candy_manager.fill_config_lines(&mut context).await.unwrap();
    candy_manager
        .set_collection(&mut context, None)
        .await
        .unwrap();

    let minter = Keypair::new();
    airdrop(&mut context, &minter.pubkey(), sol(0.01))
        .await
        .unwrap();
    candy_manager.mint_nft(&mut context, &minter).await.unwrap();
}
