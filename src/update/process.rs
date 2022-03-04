use anchor_client::{
    solana_sdk::{
        pubkey::Pubkey,
        signature::{Keypair, Signer},
    },
    Client,
};
use anchor_lang::prelude::AccountMeta;
use anyhow::Result;

use std::str::FromStr;

use mpl_candy_machine::instruction as nft_instruction;
use mpl_candy_machine::ID as CANDY_MACHINE_PROGRAM_ID;
use mpl_candy_machine::{accounts as nft_accounts, CandyMachineData};

use crate::candy_machine::*;
use crate::common::*;
use crate::config::{data::*, parser::get_config_data};
use crate::{cache::load_cache, config::data::ConfigData};

pub struct UpdateArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub new_authority: Option<String>,
    pub config: String,
}

pub fn process_update(args: UpdateArgs) -> Result<()> {
    let sugar_config = sugar_setup(args.keypair, args.rpc_url)?;
    let cache = load_cache(&args.cache)?;
    let client = setup_client(&sugar_config)?;
    let config_data = get_config_data(&args.config)?;

    let candy_machine_id = match Pubkey::from_str(&cache.program.candy_machine) {
        Ok(candy_machine_id) => candy_machine_id,
        Err(_) => {
            error!(
                "Failed to parse candy_machine_id: {}",
                &cache.program.candy_machine
            );
            std::process::exit(1);
        }
    };

    let candy_machine_state = get_candy_machine_state(&sugar_config, &candy_machine_id)?;

    let candy_machine_data = create_candy_machine_data(&config_data, candy_machine_state.data)?;

    let mut remaining_accounts: Vec<AccountMeta> = Vec::new();

    if config_data.spl_token.is_some() {
        match config_data.spl_token {
            Some(token) => remaining_accounts.push(AccountMeta {
                pubkey: token,
                is_signer: false,
                is_writable: false,
            }),
            None => (),
        }
    }

    let pid = "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ"
        .parse()
        .expect("Failed to parse PID");

    let program = client.program(pid);

    let mut builder = program
        .request()
        .accounts(nft_accounts::UpdateCandyMachine {
            candy_machine: candy_machine_id,
            authority: program.payer(),
            wallet: config_data.sol_treasury_account,
        })
        .args(nft_instruction::UpdateCandyMachine {
            data: candy_machine_data,
        });

    if !remaining_accounts.is_empty() {
        for account in remaining_accounts {
            builder = builder.accounts(account);
        }
    }

    let sig = builder.send()?;
    info!("Candy machine updated! TxId: {}", sig);

    if let Some(new_authority) = args.new_authority {
        let new_authority_pubkey = Pubkey::from_str(&new_authority)?;
        let builder = program
            .request()
            .accounts(nft_accounts::UpdateCandyMachine {
                candy_machine: candy_machine_id,
                authority: program.payer(),
                wallet: config_data.sol_treasury_account,
            })
            .args(nft_instruction::UpdateAuthority {
                new_authority: Some(new_authority_pubkey),
            });

        let sig = builder.send()?;
        info!("Candy machine update authority updated! TxId: {}", sig);
    }

    Ok(())
}

fn create_candy_machine_data(
    config: &ConfigData,
    candy_machine: CandyMachineData,
) -> Result<CandyMachineData> {
    info!("{:?}", config.go_live_date);
    let go_live_date = Some(go_live_date_as_timestamp(&config.go_live_date)?);


    let end_settings = if let Some(settings) = &config.end_settings {
        Some(settings.into_candy_format())
    } else {
        None
    };


    let whitelist_mint_settings = if let Some(settings) = &config.whitelist_mint_settings {
        Some(settings.into_candy_format())
    } else {
        None
    };


    let hidden_settings = if let Some(settings) = &config.hidden_settings {
        Some(settings.into_candy_format())
    } else {
        None
    };


    let gatekeeper = if let Some(gatekeeper) = &config.gatekeeper {
        Some(gatekeeper.into_candy_format())
    } else {
        None
    };


    let data = CandyMachineData {
        uuid: candy_machine.uuid,
        price: price_as_lamports(config.price),
        symbol: candy_machine.symbol,
        seller_fee_basis_points: candy_machine.seller_fee_basis_points,
        max_supply: config.number,
        is_mutable: config.is_mutable,
        retain_authority: config.retain_authority,
        go_live_date,
        end_settings,
        creators: candy_machine.creators,
        whitelist_mint_settings,
        hidden_settings,
        items_available: config.number,
        gatekeeper,
    };
    Ok(data)
}
