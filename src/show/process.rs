use anchor_client::solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey};
use anyhow::Result;
use chrono::NaiveDateTime;
use console::style;
use mpl_candy_machine::{EndSettingType, WhitelistMintMode};
use std::str::FromStr;

use crate::cache::load_cache;
use crate::candy_machine::*;
use crate::common::*;
use crate::utils::*;

pub struct ShowArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub candy_machine: Option<String>,
}

pub fn process_show(args: ShowArgs) -> Result<()> {
    println!(
        "{} {}Looking up candy machine",
        style("[1/1]").bold().dim(),
        LOOKING_GLASS_EMOJI
    );

    let pb = spinner_with_style();
    pb.set_message("Connecting...");

    // the candy machine id specified takes precedence over the one from the cache

    let candy_machine_id = if let Some(candy_machine) = args.candy_machine {
        candy_machine
    } else {
        let cache = load_cache(&args.cache, false)?;
        cache.program.candy_machine
    };

    let sugar_config = sugar_setup(args.keypair, args.rpc_url)?;

    let candy_machine_id = match Pubkey::from_str(&candy_machine_id) {
        Ok(candy_machine_id) => candy_machine_id,
        Err(_) => {
            let error = anyhow!("Failed to parse candy machine id: {}", candy_machine_id);
            error!("{:?}", error);
            return Err(error);
        }
    };

    let cndy_state = get_candy_machine_state(&sugar_config, &candy_machine_id)?;
    let cndy_data = cndy_state.data;

    pb.finish_and_clear();

    println!(
        "\n{}{} {}",
        CANDY_EMOJI,
        style("Candy machine ID:").dim(),
        &candy_machine_id
    );

    // candy machine state and data

    println!(" {}", style(":").dim());
    print_with_style("", "authority", cndy_state.authority.to_string());
    print_with_style("", "wallet", cndy_state.wallet.to_string());

    if let Some(token_mint) = cndy_state.token_mint {
        println!(":.. {}", token_mint);
    }

    print_with_style("", "max supply", cndy_data.max_supply.to_string());
    print_with_style("", "items redeemed", cndy_state.items_redeemed.to_string());
    print_with_style("", "items available", cndy_data.items_available.to_string());

    print_with_style("", "uuid", cndy_data.uuid.to_string());
    print_with_style(
        "",
        "price",
        format!(
            "𑗏{} ({})",
            cndy_data.price as f64 / LAMPORTS_PER_SOL as f64,
            cndy_data.price
        ),
    );
    print_with_style("", "symbol", cndy_data.symbol.to_string());
    print_with_style(
        "",
        "seller fee basis points",
        format!(
            "{}% ({})",
            cndy_data.seller_fee_basis_points / 100,
            cndy_data.seller_fee_basis_points
        ),
    );
    print_with_style("", "is mutable", cndy_data.is_mutable.to_string());
    print_with_style(
        "",
        "retain authority",
        cndy_data.retain_authority.to_string(),
    );
    if let Some(date) = cndy_data.go_live_date {
        let date = NaiveDateTime::from_timestamp(date, 0);
        print_with_style(
            "",
            "go live date",
            date.format("%a %B %e %Y %H:%M:%S UTC").to_string(),
        );
    } else {
        print_with_style("", "go live date", "none".to_string());
    }
    print_with_style("", "creators", "".to_string());

    for (index, creator) in cndy_data.creators.into_iter().enumerate() {
        let info = format!(
            "{} ({}%{})",
            creator.address,
            creator.share,
            if creator.verified { ", verified" } else { "" },
        );
        print_with_style(":   ", &(index + 1).to_string(), info);
    }

    // end settings
    if let Some(end_settings) = cndy_data.end_settings {
        print_with_style("", "end settings", "".to_string());
        match end_settings.end_setting_type {
            EndSettingType::Date => {
                print_with_style(":   ", "end setting type", "date".to_string());
                let date = NaiveDateTime::from_timestamp(end_settings.number as i64, 0);
                print_with_style(
                    ":   ",
                    "number",
                    date.format("%a %B %e %Y %H:%M:%S UTC").to_string(),
                );
            }
            EndSettingType::Amount => {
                print_with_style(":   ", "end setting type", "amount".to_string());
                print_with_style(":   ", "number", end_settings.number.to_string());
            }
        }
    } else {
        print_with_style("", "end settings", "none".to_string());
    }

    // hidden settings
    if let Some(hidden_settings) = cndy_data.hidden_settings {
        print_with_style("", "hidden settings", "".to_string());
        print_with_style(":   ", "name", hidden_settings.name);
        print_with_style(":   ", "uri", hidden_settings.uri);
        print_with_style(
            ":   ",
            "hash",
            String::from_utf8(hidden_settings.hash.to_vec())?,
        );
    } else {
        print_with_style("", "hidden settings", "none".to_string());
    }

    // whitelist mint settings
    if let Some(whitelist_settings) = cndy_data.whitelist_mint_settings {
        print_with_style("", "whiltelist mint settings", "".to_string());
        print_with_style(
            ":   ",
            "mode",
            if whitelist_settings.mode == WhitelistMintMode::BurnEveryTime {
                "burn every time".to_string()
            } else {
                "never burn".to_string()
            },
        );
        print_with_style(":   ", "mint", whitelist_settings.mint.to_string());
        print_with_style(":   ", "presale", whitelist_settings.presale.to_string());
        print_with_style(
            ":   ",
            "discount price",
            if let Some(value) = whitelist_settings.discount_price {
                format!("𑗏{} ({})", value as f64 / LAMPORTS_PER_SOL as f64, value)
            } else {
                "none".to_string()
            },
        );
    } else {
        print_with_style("", "whiltelist mint settings", "none".to_string());
    }

    // gatekeeper setttings
    if let Some(gatekeeper) = cndy_data.gatekeeper {
        print_with_style("", "gatekeeper", "".to_string());
        print_with_style(
            "    ",
            "gatekeeper network",
            gatekeeper.gatekeeper_network.to_string(),
        );
        print_with_style(
            "    ",
            "expire on use",
            gatekeeper.expire_on_use.to_string(),
        );
    } else {
        print_with_style("", "gatekeeper", "none".to_string());
    }

    Ok(())
}

fn print_with_style(indent: &str, key: &str, value: String) {
    println!(
        " {} {}",
        style(format!("{}:.. {}:", indent, key)).dim(),
        value
    );
}
