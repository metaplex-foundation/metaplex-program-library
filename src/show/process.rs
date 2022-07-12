use std::str::FromStr;

use anchor_client::solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey};
use anyhow::Result;
use chrono::NaiveDateTime;
use console::style;
use mpl_candy_machine::{utils::is_feature_active, EndSettingType, WhitelistMintMode};

use crate::{cache::load_cache, candy_machine::*, common::*, pdas::get_collection_pda, utils::*};

pub struct ShowArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub candy_machine: Option<String>,
    pub unminted: bool,
}

// TODO: change the value '1' for the corresponding constant once the
// new version of the mpl_candy_machine crate is published
const SWAP_REMOVE_FEATURE_INDEX: usize = 1;
// number of indices per line
const PER_LINE: usize = 11;

pub fn process_show(args: ShowArgs) -> Result<()> {
    println!(
        "{} {}Looking up candy machine",
        if args.unminted {
            style("[1/2]").bold().dim()
        } else {
            style("[1/1]").bold().dim()
        },
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
    let client = setup_client(&sugar_config)?;
    let program = client.program(CANDY_MACHINE_ID);

    let candy_machine_id = match Pubkey::from_str(&candy_machine_id) {
        Ok(candy_machine_id) => candy_machine_id,
        Err(_) => {
            let error = anyhow!("Failed to parse candy machine id: {}", candy_machine_id);
            error!("{:?}", error);
            return Err(error);
        }
    };

    let collection_mint =
        if let Ok((_, collection_pda)) = get_collection_pda(&candy_machine_id, &program) {
            Some(collection_pda.mint)
        } else {
            None
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
    match collection_mint {
        Some(collection_mint) => {
            print_with_style("", "collection mint", collection_mint.to_string())
        }
        None => print_with_style("", "collection mint", "none".to_string()),
    };

    if let Some(token_mint) = cndy_state.token_mint {
        print_with_style("", "spl token", token_mint.to_string());
    } else {
        print_with_style("", "spl token", "none".to_string());
    }

    print_with_style("", "max supply", cndy_data.max_supply.to_string());
    print_with_style("", "items redeemed", cndy_state.items_redeemed.to_string());
    print_with_style("", "items available", cndy_data.items_available.to_string());

    print_with_style("", "uuid", cndy_data.uuid.to_string());
    print_with_style(
        "",
        "price",
        format!(
            "◎ {} ({})",
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
        print_with_style("", "whitelist mint settings", "".to_string());
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
                format!("◎ {} ({})", value as f64 / LAMPORTS_PER_SOL as f64, value)
            } else {
                "none".to_string()
            },
        );
    } else {
        print_with_style("", "whitelist mint settings", "none".to_string());
    }

    // gatekeeper settings
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

    // unminted indices

    if args.unminted {
        println!(
            "\n{} {}Retrieving unminted indices",
            style("[2/2]").bold().dim(),
            LOOKING_GLASS_EMOJI
        );

        let mut start = CONFIG_ARRAY_START
            + STRING_LEN_SIZE
            + CONFIG_LINE_SIZE * cndy_data.items_available as usize
            + STRING_LEN_SIZE
            + cndy_data
                .items_available
                .checked_div(8)
                .expect("Numerical overflow error") as usize
            + STRING_LEN_SIZE;

        let pb = spinner_with_style();
        pb.set_message("Connecting...");
        // retrieve the (raw) candy machine data
        let data = program.rpc().get_account_data(&candy_machine_id)?;

        pb.finish_and_clear();
        let mut index = 0;
        let mut indices = vec![];

        if is_feature_active(&cndy_data.uuid, SWAP_REMOVE_FEATURE_INDEX) {
            start += 1; // needed to get around rounding precision
            let remaining = cndy_data.items_available - cndy_state.items_redeemed;
            for i in 0..remaining {
                let slice = start + (i * 4) as usize;
                indices.push(u32::from_le_bytes(
                    data[slice..slice + 4].try_into().unwrap(),
                ));
            }
        } else {
            while start < data.len() {
                let mask = 1u8 << 7;

                for i in 0..8 {
                    if index < cndy_data.items_available {
                        // unused mint indices have the 'flag' set to 0
                        if (data[start] & (mask >> i)) == 0 {
                            indices.push(index as u32);
                        }
                        index += 1;
                    }
                }

                start += 1;
            }
        }

        if indices.is_empty() {
            println!(
                "\n{}{}",
                PAPER_EMOJI,
                style("All items of the candy machine have been minted.").dim()
            );
        } else {
            // makes sure all items are in order
            indices.sort_unstable();
            // logs all indices
            info!("unminted list: {:?}", indices);

            println!(
                "\n{}{}",
                PAPER_EMOJI,
                style(format!("Unminted list ({} total):", indices.len())).dim()
            );
            let mut current = 0;

            for i in indices {
                if current == 0 {
                    println!("{}", style(" :").dim());
                    print!("{}", style(" :.. ").dim());
                }
                current += 1;

                print!(
                    "{:<5}{}",
                    i,
                    if current == PER_LINE {
                        current = 0;
                        "\n"
                    } else {
                        " "
                    }
                );
            }
            // just adds a new line break
            println!();
        }
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
