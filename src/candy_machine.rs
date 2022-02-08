#![allow(unused)]
use anchor_client::{
    solana_sdk::{
        borsh::try_from_slice_unchecked,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        system_instruction, system_program, sysvar,
    },
    Client,
};
use anchor_lang::AccountDeserialize;
use anyhow::Result;
use chrono::naive::serde::ts_milliseconds::deserialize;
use rand::rngs::OsRng;
use rayon::prelude::*;
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
};

use mpl_candy_machine::accounts as nft_accounts;
use mpl_candy_machine::instruction as nft_instruction;
use mpl_candy_machine::{
    CandyMachine, CandyMachineData, ConfigLine, Creator as CandyCreator, WhitelistMintMode,
    WhitelistMintSettings,
};

use crate::config::data::SugarConfig;
use crate::setup::setup_client;

#[derive(Debug)]
pub struct ConfigStatus {
    pub index: u32,
    pub on_chain: bool,
}

pub fn get_candy_machine_state(
    sugar_config: &SugarConfig,
    candy_machine_id: &Pubkey,
) -> Result<CandyMachine> {
    let client = setup_client(sugar_config)?;
    let pid = "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ"
        .parse()
        .expect("Failed to parse PID");

    let program = client.program(pid);
    let mut data = program.rpc().get_account_data(candy_machine_id)?;
    let candy_machine: CandyMachine = CandyMachine::try_deserialize(&mut data.as_slice())?;
    Ok(candy_machine)
}

pub fn get_candy_machine_data(
    sugar_config: &SugarConfig,
    candy_machine_id: &Pubkey,
) -> Result<CandyMachineData> {
    let client = setup_client(sugar_config)?;
    let pid = "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ"
        .parse()
        .expect("Failed to parse PID");

    let program = client.program(pid);
    let mut data = program.rpc().get_account_data(candy_machine_id)?;
    let candy_machine: CandyMachine = CandyMachine::try_deserialize(&mut data.as_slice())?;
    Ok(candy_machine.data)
}

pub fn uuid_from_pubkey(pubkey: &Pubkey) -> String {
    pubkey.to_string()[0..6].to_string()
}

pub fn print_candy_machine_state(state: CandyMachine) {
    println!("Authority {:?}", state.authority);
    println!("Wallet {:?}", state.wallet);
    println!("Token mint: {:?}", state.token_mint);
    println!("Items redeemed: {:?}", state.items_redeemed);
    print_candy_machine_data(&state.data);
}

pub fn print_candy_machine_data(data: &CandyMachineData) {
    println!("Uuid: {:?}", data.uuid);
    println!("Price: {:?}", data.price);
    println!("Symbol: {:?}", data.symbol);
    println!(
        "Seller fee basis points: {:?}",
        data.seller_fee_basis_points
    );
    println!("Max supply: {:?}", data.max_supply);
    println!("Is mutable: {:?}", data.is_mutable);
    println!("Retain Authority: {:?}", data.retain_authority);
    println!("Go live date: {:?}", data.go_live_date);
    println!("Items available: {:?}", data.items_available);

    print_whitelist_mint_settings(&data.whitelist_mint_settings);
}

fn print_whitelist_mint_settings(settings: &Option<WhitelistMintSettings>) {
    if let Some(settings) = settings {
        match settings.mode {
            WhitelistMintMode::BurnEveryTime => println!("Mode: Burn every time"),
            WhitelistMintMode::NeverBurn => println!("Mode: Never burn"),
        }
        println!("Mint: {:?}", settings.mint);
        println!("Presale: {:?}", settings.presale);
        println!("Discount price: {:?}", settings.discount_price);
    } else {
        println!("No whitelist mint settings");
    }
}
