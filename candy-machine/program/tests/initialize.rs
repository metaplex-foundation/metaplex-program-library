#![cfg(feature = "test-bpf")]

use std::assert_eq;

use anchor_lang::AccountDeserialize;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

use mpl_candy_machine::{CandyMachine, CandyMachineData};
use utils::setup_functions;

use crate::{core::airdrop, utils::add_all_config_lines};

mod core;
pub mod utils;

#[tokio::test]
async fn init_default_success() {
    let mut context = setup_functions::candy_machine_program_test()
        .start_with_context()
        .await;
    let payer_wallet = Keypair::new();

    airdrop(&mut context, &payer_wallet.pubkey(), 10_000_000_000)
        .await
        .unwrap();

    let authority = Keypair::new();
    airdrop(&mut context, &authority.pubkey(), 10_000_000_000)
        .await
        .unwrap();

    let wallet = Keypair::new();
    let candy_data = CandyMachineData {
        uuid: "123456".to_string(),
        items_available: 995,
        ..CandyMachineData::default()
    };
    let wallet_key = &wallet.pubkey();

    let candy_key =
        setup_functions::create_candy_machine(&mut context, wallet_key, candy_data.clone(), None)
            .await
            .unwrap();

    add_all_config_lines(&mut context, &candy_key)
        .await
        .unwrap();

    let candy_machine_account = context
        .banks_client
        .get_account(candy_key)
        .await
        .expect("account not found")
        .expect("account empty");

    let candy_machine_data: CandyMachine =
        CandyMachine::try_deserialize(&mut candy_machine_account.data.as_ref()).unwrap();

    println!(
        "{:?}",
        String::from_utf8_lossy(candy_machine_account.data.as_slice())
    );
    println!("{:?}", candy_machine_account.data.as_slice());
    println!("{:?}", candy_machine_data);

    assert_eq!(&candy_machine_data.data.uuid, &candy_data.uuid);
    assert_eq!(
        &candy_machine_data.data.seller_fee_basis_points,
        &candy_data.seller_fee_basis_points
    );
    assert_eq!(
        &candy_machine_data.data.creators.len(),
        &candy_data.creators.len()
    );
    assert_eq!(&candy_machine_data.data.price, &candy_data.price);
    assert_eq!(
        &candy_machine_data.data.items_available,
        &candy_data.items_available
    );
}
