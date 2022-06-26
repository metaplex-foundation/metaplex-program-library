#![cfg(feature = "test-bpf")]

use anchor_lang::{
    error, AnchorDeserialize, AnchorSerialize, InstructionData, Key, ToAccountMetas,
};
use borsh::BorshDeserialize;
use std::{
    convert::TryInto,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use solana_gateway::{
    borsh::{self as program_borsh, get_instance_packed_len},
    instruction::{self, NetworkFeature},
    state::{
        get_expire_address_with_seed, get_gatekeeper_address_with_seed,
        get_gateway_token_address_with_seed, GatewayToken, GATEWAY_TOKEN_ADDRESS_SEED,
    },
    Gateway,
};
use solana_gateway_program::processor::process_instruction;
use solana_program_test::*;
use solana_sdk::{
    account_info::AccountInfo,
    borsh::try_from_slice_unchecked,
    clock::UnixTimestamp,
    instruction::{AccountMeta, Instruction, InstructionError},
    msg,
    program::invoke_signed,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction, system_program, sysvar,
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};

use crate::{
    core::helpers::{airdrop, get_account, get_network_expire, get_network_token, prepare_nft},
    utils::{
        auto_config, candy_machine_program_test, helpers::sol, CandyManager, GatekeeperConfig,
        WhitelistConfig,
    },
};
use mpl_candy_machine::{
    constants::{BOT_FEE, EXPIRE_OFFSET},
    punish_bots, CandyError, CandyMachineData, GatekeeperConfig as GKConfig,
    WhitelistMintMode::BurnEveryTime,
    WhitelistMintSettings,
};
use utils::{custom_config, helpers::find_candy_creator, GatekeeperInfo};

mod core;
mod utils;

// #[tokio::test]
// async fn init_default_success() {
//     let mut context = candy_machine_program_test().start_with_context().await;
//     let context = &mut context;

//     let mut &candy_manager = CandyManager::init(
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
    let mut program = ProgramTest::new("mpl_candy_machine", mpl_candy_machine::id(), None);
    program.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
    program.add_program(
        "solana_gateway_program",
        Pubkey::from_str("gatem74V238djXdzWnJf94Wo1DcnuGkfijbf3AuBhfs").unwrap(),
        None,
    );

    let mut context = program.start_with_context().await;
    let context = &mut context;

    let gatekeeper_network = Keypair::new();
    let gatekeeper_authority = Keypair::new();

    let mut candy_manager = CandyManager::init(
        context,
        false,
        false,
        None,
        Some(GatekeeperInfo {
            set: true,
            network_expire_feature: None,
            gateway_app: Pubkey::from_str("gatem74V238djXdzWnJf94Wo1DcnuGkfijbf3AuBhfs").unwrap(),
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

    let transaction = Transaction::new_signed_with_payer(
        &[instruction::add_gatekeeper(
            &context.payer.pubkey(),
            &gatekeeper_authority.pubkey(),
            &gatekeeper_network.pubkey(),
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &gatekeeper_network],
        context.last_blockhash,
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
        context.last_blockhash,
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

    candy_manager.mint_and_assert_bot_tax(context).await.unwrap();
}
