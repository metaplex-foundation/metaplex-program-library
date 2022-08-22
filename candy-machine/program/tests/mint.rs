#![cfg(feature = "test-bpf")]
#![allow(dead_code)]

use solana_program::instruction::InstructionError;
use solana_program_test::*;
use solana_sdk::account::{AccountSharedData, WritableAccount};
use solana_sdk::transaction::TransactionError;
use solana_sdk::transport::TransportError;
use solana_sdk::{signature::Keypair, signer::Signer};

use crate::core::helpers::prepare_nft;
use crate::utils::helpers::find_candy_creator;
use crate::utils::mint_nft;
use crate::{
    core::helpers::airdrop,
    utils::{auto_config, candy_machine_program_test, helpers::sol, CandyManager},
};

pub mod core;
pub mod utils;

#[tokio::test]
async fn fail_metadata_not_blank() {
    let mut context = candy_machine_program_test().start_with_context().await;
    let context = &mut context;
    let mut candy_manager = CandyManager::init(context, false, false, None, None, None).await;

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

    let nft_info = prepare_nft(context, &candy_manager.minter).await;

    context.set_account(
        &nft_info.metadata_pubkey,
        &AccountSharedData::create(
            1000000000,
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1],
            mpl_token_metadata::id(),
            false,
            1,
        ),
    );
    let (candy_machine_creator, creator_bump) =
        find_candy_creator(&candy_manager.candy_machine.pubkey());
    let err = match mint_nft(
        context,
        &candy_manager.candy_machine.pubkey(),
        &candy_machine_creator,
        creator_bump,
        &candy_manager.wallet,
        &candy_manager.authority.pubkey(),
        &candy_manager.minter,
        &nft_info,
        candy_manager.token_info.clone(),
        candy_manager.whitelist_info.clone(),
        candy_manager.collection_info.clone(),
        candy_manager.gateway_info.clone(),
        candy_manager.freeze_info.clone(),
    )
    .await
    .unwrap_err()
    {
        TransportError::TransactionError(TransactionError::InstructionError(
            0,
            InstructionError::Custom(err_num),
        )) => err_num,
        _ => 0,
    };
    assert_eq!(err, 6031)
}
