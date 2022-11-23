#![cfg(feature = "test-bpf")]
#![allow(dead_code)]

use anchor_client::solana_sdk::transaction::Transaction;
use mpl_candy_machine::WhitelistMintMode;
use solana_program::{instruction::InstructionError, pubkey::Pubkey};
use solana_program_test::*;
use solana_sdk::{
    account::{AccountSharedData, WritableAccount},
    signature::Keypair,
    signer::Signer,
    transaction::TransactionError,
};

use crate::{
    core::helpers::{airdrop, prepare_nft, update_blockhash},
    utils::{
        auto_config, candy_machine_program_test,
        helpers::{find_candy_creator, sol, test_start},
        mint_nft, mint_nft_ix, CandyManager, WhitelistConfig, WhitelistInfo,
    },
};

pub mod core;
pub mod utils;

#[tokio::test]
async fn fail_metadata_not_blank() {
    test_start("Fail Metadata Not Blank");
    let mut context = candy_machine_program_test().start_with_context().await;
    let context = &mut context;
    let mut candy_manager = CandyManager::init(context, None, false, None, None, None).await;

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
        BanksClientError::TransactionError(TransactionError::InstructionError(
            0,
            InstructionError::Custom(err_num),
        )) => err_num,
        _ => 0,
    };
    assert_eq!(err, 6031)
}

#[tokio::test]
async fn metadata_check_before_bot_tax() {
    test_start("Metadata Check Before Bot Tax");
    let mut context = candy_machine_program_test().start_with_context().await;
    let context = &mut context;
    let mut candy_manager = CandyManager::init(context, None, false, None, None, None).await;

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
    candy_manager.whitelist_info = WhitelistInfo {
        set: true,
        mint: Pubkey::new_unique(),
        auth_account: Pubkey::new_unique(),
        minter_account: Pubkey::new_unique(),
        whitelist_config: WhitelistConfig {
            burn: WhitelistMintMode::BurnEveryTime,
            presale: false,
            discount_price: None,
        },
    };

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
    let mut ix = mint_nft_ix(
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
    );

    ix[0].accounts.pop();
    update_blockhash(context).await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        ix.as_slice(),
        Some(&candy_manager.minter.pubkey()),
        &[&candy_manager.minter],
        context.last_blockhash,
    );

    let err = match context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err()
    {
        BanksClientError::TransactionError(TransactionError::InstructionError(
            0,
            InstructionError::Custom(err_num),
        )) => err_num,
        _ => 0,
    };
    assert_eq!(err, 6031)
}
