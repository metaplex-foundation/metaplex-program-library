use std::str::FromStr;

use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_lang::prelude::AccountMeta;
use anyhow::Result;
use chrono::Utc;
use console::{style, Style};
use dialoguer::{theme::ColorfulTheme, Confirm};
use mpl_candy_machine::{
    accounts as nft_accounts, instruction as nft_instruction, CandyMachineData,
};
use spl_associated_token_account::get_associated_token_address;

use crate::{
    cache::load_cache,
    candy_machine::{get_candy_machine_state, parse_config_price, CANDY_MACHINE_ID},
    common::*,
    config::{
        data::{ConfigData, *},
        parser::get_config_data,
    },
    utils::{
        assert_correct_authority, check_spl_token, check_spl_token_account, get_dialoguer_theme,
        get_mint_decimals, spinner_with_style,
    },
};

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

    if let Err(e) = check_config_to_prevent_bots(&config_data) {
        println!("{WARNING_EMOJI} {e}");
        return Ok(());
    };

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

    println!(
        "{} {}Loading candy machine",
        style("[1/2]").bold().dim(),
        LOOKING_GLASS_EMOJI
    );
    println!("{} {}", style("Candy machine ID:").bold(), candy_machine_id);

    let pb = spinner_with_style();
    pb.set_message("Connecting...");

    let candy_machine_state = get_candy_machine_state(&sugar_config, &candy_pubkey)?;
    let candy_machine_data =
        create_candy_machine_data(&client, &config_data, &candy_machine_state.data)?;

    pb.finish_with_message("Done");

    assert_correct_authority(
        &sugar_config.keypair.pubkey(),
        &candy_machine_state.authority,
    )?;

    println!(
        "\n{} {}Updating configuration",
        style("[2/2]").bold().dim(),
        COMPUTER_EMOJI
    );

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

    let program = client.program(CANDY_MACHINE_ID);

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

    let pb = spinner_with_style();
    pb.set_message("Sending update transaction...");

    let update_signature = builder.send()?;

    pb.finish_with_message(format!(
        "{} {}",
        style("Update signature:").bold(),
        update_signature
    ));

    if let Some(new_authority) = args.new_authority {
        let pb = spinner_with_style();
        pb.set_message("Sending update authority transaction...");

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

        let authority_signature = builder.send()?;
        pb.finish_with_message(format!(
            "{} {}",
            style("Authority signature:").bold(),
            authority_signature
        ));
    }

    Ok(())
}

fn create_candy_machine_data(
    client: &Client,
    config: &ConfigData,
    candy_machine: &CandyMachineData,
) -> Result<CandyMachineData> {
    let program = client.program(CANDY_MACHINE_ID);

    info!("{:?}", config.go_live_date);
    let go_live_date: Option<i64> = go_live_date_as_timestamp(&config.go_live_date)?;

    let end_settings = if let Some(end_settings) = &config.end_settings {
        Some(end_settings.to_candy_format()?)
    } else {
        None
    };

    // If SPL token is used, get the decimals from the token mint account, otherwise use 9 for SOL.
    let decimals = get_mint_decimals(&program, config)?;

    let whitelist_mint_settings = config
        .whitelist_mint_settings
        .as_ref()
        .map(|s| s.to_candy_format(decimals));

    let hidden_settings = config.hidden_settings.as_ref().map(|s| s.to_candy_format());

    let gatekeeper = config.gatekeeper.as_ref().map(|g| g.to_candy_format());

    let price = parse_config_price(client, config)?;

    let creators = config
        .creators
        .clone()
        .into_iter()
        .map(|c| c.to_candy_format())
        .collect::<Result<Vec<mpl_candy_machine::Creator>>>()?;

    let data = CandyMachineData {
        uuid: candy_machine.uuid.clone(),
        price,
        symbol: config.symbol.clone(),
        seller_fee_basis_points: config.seller_fee_basis_points,
        max_supply: 0,
        is_mutable: config.is_mutable,
        retain_authority: config.retain_authority,
        go_live_date,
        end_settings,
        creators,
        whitelist_mint_settings,
        hidden_settings,
        items_available: config.number,
        gatekeeper,
    };
    Ok(data)
}

// warn user and prompt for confirmation if
//   - price is 0
//   - no gatekeeper is set
//   - no whitelist token is set
// if go live date is absent or less than 1 week from now prompt, else warn only
pub fn check_config_to_prevent_bots(config_data: &ConfigData) -> Result<(), String> {
    let now = Utc::now().timestamp();
    let time_to_go_live =
        match go_live_date_as_timestamp(&config_data.go_live_date).map_err(|e| e.to_string())? {
            Some(time) => time - now,
            None => 0,
        };

    const ONE_MINUTE: i64 = 60;
    const ONE_HOUR: i64 = ONE_MINUTE * 60;
    const ONE_DAY: i64 = ONE_HOUR * 24;
    const ONE_WEEK: i64 = ONE_DAY * 7;

    let time_left = if time_to_go_live < 0 {
        "CURRENTLY LIVE".to_string()
    } else {
        let days = time_to_go_live / ONE_DAY;
        let hours = (time_to_go_live % ONE_DAY) / ONE_HOUR;
        let minutes = (time_to_go_live % ONE_HOUR) / ONE_MINUTE;
        let seconds = time_to_go_live % ONE_MINUTE;
        format!("in {days}d {hours}h {minutes}m {seconds}s")
    };

    // invert conditions to allow returning early
    if config_data.price > 0f64
        || config_data.gatekeeper.is_some()
        || config_data.whitelist_mint_settings.is_some()
    {
        Ok(())
    } else {
        println!("{WARNING_EMOJI} warning: the candy machine will be created with 0 price and no gatekeeper or whitelist token");
        println!(
            "{WARNING_EMOJI} bots will be able to mint from it as soon as it goes live on {} ({})",
            config_data
                .go_live_date
                .as_ref()
                .unwrap_or(&"-".to_string()),
            time_left
        );
        println!();

        if time_to_go_live > ONE_WEEK {
            return Ok(());
        } else {
            println!("{WARNING_EMOJI} candy machine has less than one week left before going live");
        }

        println!();
        println!(
            "{WARNING_EMOJI} it is highly recommended you quit and fix the config before proceeding");

        let theme = ColorfulTheme {
            success_prefix: style("✔".to_string()).yellow().force_styling(true),
            values_style: Style::new().yellow(),
            ..get_dialoguer_theme()
        };
        if !Confirm::with_theme(&theme)
            .with_prompt("are you sure you want to continue?")
            .interact()
            .map_err(|e| e.to_string())?
        {
            Err("action aborted".to_string())
        } else {
            Ok(())
        }
    }
}
