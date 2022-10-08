use std::str::FromStr;

use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use console::style;
use mpl_candy_machine_core::constants::NULL_STRING;

use crate::{cache::load_cache, candy_machine::*, common::*, utils::*};

pub struct ShowArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub candy_machine: Option<String>,
    pub unminted: bool,
}

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
    print_with_style("", "mint authority", cndy_state.mint_authority.to_string());
    print_with_style(
        "",
        "collection mint",
        cndy_state.collection_mint.to_string(),
    );

    print_with_style("", "max supply", cndy_data.max_supply.to_string());
    print_with_style("", "items redeemed", cndy_state.items_redeemed.to_string());
    print_with_style("", "items available", cndy_data.items_available.to_string());

    if cndy_state.features.count_ones() > 0 {
        print_with_style("", "features", cndy_state.features);
    } else {
        print_with_style("", "features", "none");
    }

    print_with_style("", "symbol", cndy_data.symbol.trim_end_matches(NULL_STRING));
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
    print_with_style("", "creators", "".to_string());

    let creators = &cndy_data.creators;

    for (index, creator) in creators.iter().enumerate() {
        let info = format!(
            "{} ({}%{})",
            creator.address,
            creator.percentage_share,
            if creator.verified { ", verified" } else { "" },
        );
        print_with_style(":   ", &(index + 1).to_string(), info);
    }

    // hidden settings

    if let Some(hidden_settings) = &cndy_data.hidden_settings {
        print_with_style("", "hidden settings", "".to_string());
        print_with_style(":   ", "name", &hidden_settings.name);
        print_with_style(":   ", "uri", &hidden_settings.uri);
        print_with_style(
            ":   ",
            "hash",
            String::from_utf8(hidden_settings.hash.to_vec())?,
        );
    } else {
        print_with_style("", "hidden settings", "none".to_string());
    }

    // config line settings

    if let Some(config_line_settings) = &cndy_data.config_line_settings {
        print_with_style("", "config line settings", "");

        let prefix_name = if config_line_settings.prefix_name.is_empty() {
            style("<empty>").dim()
        } else {
            style(config_line_settings.prefix_name.as_str())
        };
        print_with_style("    ", "prefix_name", &prefix_name.to_string());
        print_with_style(
            "    ",
            "name_length",
            &config_line_settings.name_length.to_string(),
        );

        let prefix_uri = if config_line_settings.prefix_uri.is_empty() {
            style("<empty>").dim()
        } else {
            style(config_line_settings.prefix_uri.as_str())
        };
        print_with_style("    ", "prefix_uri", &prefix_uri.to_string());
        print_with_style(
            "    ",
            "uri_length",
            &config_line_settings.uri_length.to_string(),
        );
        print_with_style(
            "    ",
            "is_sequential",
            if config_line_settings.is_sequential {
                "true"
            } else {
                "false"
            },
        );
    } else {
        print_with_style("", "config line settings", "none");
    }

    // unminted indices

    if args.unminted {
        println!(
            "\n{} {}Retrieving unminted indices",
            style("[2/2]").bold().dim(),
            LOOKING_GLASS_EMOJI
        );

        let start = CONFIG_ARRAY_START
            + STRING_LEN_SIZE
            + (cndy_data.items_available as usize * cndy_data.get_config_line_size())
            + cndy_data
                .items_available
                .checked_div(8)
                .expect("Numerical overflow error") as usize
            + 1;

        let pb = spinner_with_style();
        pb.set_message("Connecting...");
        // retrieve the (raw) candy machine data
        let data = program.rpc().get_account_data(&candy_machine_id)?;

        pb.finish_and_clear();
        let mut indices = vec![];

        let remaining = cndy_data.items_available - cndy_state.items_redeemed;
        for i in 0..remaining {
            let slice = start + (i * 4) as usize;
            indices.push(u32::from_le_bytes(
                data[slice..slice + 4].try_into().unwrap(),
            ));
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

fn print_with_style<S>(indent: &str, key: &str, value: S)
where
    S: core::fmt::Display,
{
    println!(
        " {} {}",
        style(format!("{}:.. {}:", indent, key)).dim(),
        value
    );
}
