use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_lang::prelude::AccountMeta;
use anyhow::Result;
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;

use mpl_candy_machine::instruction as nft_instruction;
use mpl_candy_machine::{accounts as nft_accounts, CandyMachineData};

use crate::candy_machine::*;
use crate::common::*;
use crate::config::{data::*, parser::get_config_data};
use crate::constants::CANDY_MACHINE_V2;
use crate::utils::{check_spl_token, check_spl_token_account};
use crate::{cache::load_cache, config::data::ConfigData};

pub struct UpdateArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub new_authority: Option<String>,
    pub config: String,
    pub candy_machine: Option<String>,
}

pub fn process_update(args: UpdateArgs) -> Result<()> {
    let sugar_config = sugar_setup(args.keypair, args.rpc_url)?;
    let client = setup_client(&sugar_config)?;
    let config_data = get_config_data(&args.config)?;

    // the candy machine id specified takes precedence over the one from the cache

    let candy_machine_id = match args.candy_machine {
        Some(candy_machine_id) => candy_machine_id,
        None => {
            let cache = load_cache(&args.cache, false)?;
            cache.program.candy_machine
        }
    };

    let candy_pubkey = match Pubkey::from_str(&candy_machine_id) {
        Ok(candy_pubkey) => candy_pubkey,
        Err(_) => {
            let error = anyhow!("Failed to parse candy machine id: {}", candy_machine_id);
            error!("{:?}", error);
            return Err(error);
        }
    };

    let candy_machine_state = get_candy_machine_state(&sugar_config, &candy_pubkey)?;
    let candy_machine_data = create_candy_machine_data(&config_data, candy_machine_state.data)?;

    let mut remaining_accounts: Vec<AccountMeta> = Vec::new();

    if config_data.spl_token.is_some() {
        if let Some(token) = config_data.spl_token {
            remaining_accounts.push(AccountMeta {
                pubkey: token,
                is_signer: false,
                is_writable: false,
            })
        }
    }

    let pid = CANDY_MACHINE_V2.parse().expect("Failed to parse PID");

    let program = client.program(pid);

    let treasury_account = match config_data.spl_token {
        Some(spl_token) => {
            let spl_token_account_figured = if config_data.spl_token_account.is_some() {
                config_data.spl_token_account
            } else {
                Some(get_associated_token_address(&program.payer(), &spl_token))
            };

            if config_data.sol_treasury_account.is_some() {
                return Err(anyhow!("If spl-token-account or spl-token is set then sol-treasury-account cannot be set"));
            }

            // validates the mint address of the token accepted as payment
            check_spl_token(&program, &spl_token.to_string())?;

            if let Some(token_account) = spl_token_account_figured {
                // validates the spl token wallet to receive proceedings from SPL token payments
                check_spl_token_account(&program, &token_account.to_string())?;
                token_account
            } else {
                return Err(anyhow!(
                    "If spl-token is set, spl-token-account must also be set"
                ));
            }
        }
        None => match config_data.sol_treasury_account {
            Some(sol_treasury_account) => sol_treasury_account,
            None => sugar_config.keypair.pubkey(),
        },
    };

    let mut builder = program
        .request()
        .accounts(nft_accounts::UpdateCandyMachine {
            candy_machine: candy_pubkey,
            authority: program.payer(),
            wallet: treasury_account,
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
    let message = format!("Candy machine updated.\nSignature: {sig}");
    info!("{message}");
    println!("{message}");

    if let Some(new_authority) = args.new_authority {
        let new_authority_pubkey = Pubkey::from_str(&new_authority)?;

        let builder = program
            .request()
            .accounts(nft_accounts::UpdateCandyMachine {
                candy_machine: candy_pubkey,
                authority: program.payer(),
                wallet: treasury_account,
            })
            .args(nft_instruction::UpdateAuthority {
                new_authority: Some(new_authority_pubkey),
            });

        let sig = builder.send()?;
        let message = format!("Candy machine update authority updated.\nSignature: {sig}");
        info!("{message}");
        println!("{message}");
    }

    Ok(())
}

fn create_candy_machine_data(
    config: &ConfigData,
    candy_machine: CandyMachineData,
) -> Result<CandyMachineData> {
    info!("{:?}", config.go_live_date);
    let go_live_date = Some(go_live_date_as_timestamp(&config.go_live_date)?);

    let end_settings = &config.end_settings.as_ref().map(|s| s.into_candy_format());

    let whitelist_mint_settings = &config
        .whitelist_mint_settings
        .as_ref()
        .map(|s| s.into_candy_format());

    let hidden_settings = &config
        .hidden_settings
        .as_ref()
        .map(|s| s.into_candy_format());

    let gatekeeper = config.gatekeeper.as_ref().map(|g| g.into_candy_format());

    let data = CandyMachineData {
        uuid: candy_machine.uuid,
        price: price_as_lamports(config.price),
        symbol: candy_machine.symbol,
        seller_fee_basis_points: candy_machine.seller_fee_basis_points,
        max_supply: 0,
        is_mutable: config.is_mutable,
        retain_authority: config.retain_authority,
        go_live_date,
        end_settings: end_settings.clone(),
        creators: candy_machine.creators,
        whitelist_mint_settings: whitelist_mint_settings.clone(),
        hidden_settings: hidden_settings.clone(),
        items_available: config.number,
        gatekeeper,
    };
    Ok(data)
}
