#![cfg(feature = "test-bpf")]

use std::str::FromStr;

use anchor_lang::{AnchorDeserialize, InstructionData, ToAccountMetas};
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction, InstructionError},
    msg,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program, sysvar,
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};

use crate::{
    core::helpers::{airdrop, get_network_expire, get_network_token, prepare_nft},
    utils::{auto_config, candy_machine_program_test, helpers::sol, CandyManager, WhitelistConfig},
};
use mpl_candy_machine::{CandyMachineData, GatekeeperConfig, WhitelistMintMode::BurnEveryTime};
use utils::{custom_config, helpers::find_candy_creator};

mod core;
mod utils;

// #[tokio::test]
// async fn init_default_success() {
//     let mut context = candy_machine_program_test().start_with_context().await;
//     let context = &mut context;

//     let mut candy_manager = CandyManager::init(
//         context,
//         true,
//         true,
//         Some(WhitelistConfig::new(BurnEveryTime, false, Some(1))),
//     )
//     .await;

//     airdrop(context, &candy_manager.minter.pubkey(), sol(2.0))
//         .await
//         .unwrap();
//     let candy_data = auto_config(&&candy_manager, None, true, true, None, None);
//     candy_manager
//         .create(context, candy_data.clone())
//         .await
//         .unwrap();
//     candy_manager.fill_config_lines(context).await.unwrap();
//     candy_manager.set_collection(context).await.unwrap();

//     let failed = candy_manager.mint_and_assert_bot_tax(context).await;
//     if failed.is_err() {
//         println!("Had an error when it potentially should have been bot tax!");
//     }
//     let candy_data = CandyMachineData {
//         go_live_date: Some(0),
//         price: 1,
//         ..candy_data
//     };
//     candy_manager
//         .update(context, None, candy_data)
//         .await
//         .unwrap();
//     candy_manager
//         .mint_and_assert_successful(context, Some(1), true)
//         .await;
// }

#[tokio::test]
async fn bot_tax_on_gatekeeper() {
    let mut context = candy_machine_program_test().start_with_context().await;
    let context = &mut context;

    let mut candy_manager = CandyManager::init(context, true, false, None).await;

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
        Some(GatekeeperConfig {
            gatekeeper_network: Pubkey::from_str("ignREusXmGrscGNUesoU9mxfds9AiYTezUKex2PsZV6")
                .unwrap(),
            expire_on_use: true,
        }),
    );

    candy_manager
        .create(context, candy_data.clone())
        .await
        .unwrap();
    candy_manager.fill_config_lines(context).await.unwrap();

    let nft_info = prepare_nft(context, &candy_manager.minter).await;
    let (candy_machine_creator, creator_bump) =
        find_candy_creator(&candy_manager.candy_machine.pubkey());

    let metadata = nft_info.metadata_pubkey;
    let master_edition = nft_info.pubkey;
    let mint = nft_info.mint_pubkey;

    let network_token = get_network_token(
        &candy_manager.minter.pubkey(),
        &Pubkey::from_str("ignREusXmGrscGNUesoU9mxfds9AiYTezUKex2PsZV6").unwrap(),
        Pubkey::from_str("gatem74V238djXdzWnJf94Wo1DcnuGkfijbf3AuBhfs").unwrap(),
    );

    let expire_token = get_network_expire(
        &Pubkey::from_str("ignREusXmGrscGNUesoU9mxfds9AiYTezUKex2PsZV6").unwrap(),
        Pubkey::from_str("gatem74V238djXdzWnJf94Wo1DcnuGkfijbf3AuBhfs").unwrap(),
    );

    // inserting incorrect program_id
    let program_id = Pubkey::from_str("hater74V238djXdzWnJf94Wo1DcnuGkfijbf3AuBhfs").unwrap();

    let mut accounts = mpl_candy_machine::accounts::MintNFT {
        candy_machine: candy_manager.candy_machine.pubkey(),
        candy_machine_creator: candy_machine_creator,
        payer: candy_manager.minter.pubkey(),
        wallet: candy_manager.wallet,
        metadata,
        mint,
        mint_authority: candy_manager.minter.pubkey(),
        update_authority: candy_manager.minter.pubkey(),
        master_edition,
        token_metadata_program: mpl_token_metadata::id(),
        token_program: spl_token::id(),
        system_program: system_program::id(),
        rent: sysvar::rent::id(),
        clock: sysvar::clock::id(),
        recent_blockhashes: sysvar::slot_hashes::id(),
        instruction_sysvar_account: sysvar::instructions::id(),
    }
    .to_account_metas(None);

    accounts.push(AccountMeta::new(network_token.0, false));
    accounts.push(AccountMeta::new_readonly(expire_token.0, false));
    accounts.push(AccountMeta::new_readonly(program_id, false));

    let data = mpl_candy_machine::instruction::MintNft { creator_bump }.data();

    let mut instructions = Vec::new();
    let mut signers = Vec::new();

    let mint_ix = Instruction {
        program_id: mpl_candy_machine::id(),
        data,
        accounts,
    };
    instructions.push(mint_ix);
    signers.push(&candy_manager.minter);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&candy_manager.minter.pubkey()),
        &signers,
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // match err {
    //     TransportError::TransactionError(TransactionError::Error(
    //         0,
    //         InstructionError::Custom(6000),
    //     )) => (),
    //     _ => panic!("Expected custom error"),
    // }
}
