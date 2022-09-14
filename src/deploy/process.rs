use std::{
    collections::HashSet,
    fmt::Write as _,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anchor_client::solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use anyhow::Result;
use console::style;
use mpl_candy_machine::{constants::FREEZE_FEATURE_INDEX, utils::is_feature_active};
use spl_associated_token_account::get_associated_token_address;

use crate::{
    cache::*,
    candy_machine::{get_candy_machine_state, CANDY_MACHINE_ID},
    common::*,
    config::parser::get_config_data,
    deploy::{
        create_and_set_collection, create_candy_machine_data, errors::*, generate_config_lines,
        initialize_candy_machine, upload_config_lines,
    },
    freeze::enable_freeze,
    hash::hash_and_update,
    setup::{setup_client, sugar_setup},
    update::{process_update, UpdateArgs},
    utils::*,
    validate::parser::{check_name, check_seller_fee_basis_points, check_symbol, check_url},
};

pub struct DeployArgs {
    pub config: String,
    pub cache: String,
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub interrupted: Arc<AtomicBool>,
}

pub async fn process_deploy(args: DeployArgs) -> Result<()> {
    // loads the cache file (this needs to have been created by
    // the upload command)
    let mut cache = load_cache(&args.cache, false)?;

    if cache.items.is_empty() {
        println!(
            "{}",
            style("No cache items found - run 'upload' to create the cache file first.")
                .red()
                .bold()
        );

        // nothing else to do, just tell that the cache file was not found (or empty)
        return Err(CacheError::CacheFileNotFound(args.cache).into());
    }

    // checks that all metadata information are present and have the
    // correct length

    for (index, item) in &cache.items.0 {
        if item.name.is_empty() {
            return Err(DeployError::MissingName(index.to_string()).into());
        } else {
            check_name(&item.name)?;
        }

        if item.metadata_link.is_empty() {
            return Err(DeployError::MissingMetadataLink(index.to_string()).into());
        } else {
            check_url(&item.metadata_link)?;
        }
    }

    let sugar_config = Arc::new(sugar_setup(args.keypair.clone(), args.rpc_url.clone())?);
    let client = setup_client(&sugar_config)?;
    let program = client.program(CANDY_MACHINE_ID);
    let mut config_data = get_config_data(&args.config)?;

    let candy_machine_address = &cache.program.candy_machine;

    // checks the candy machine data

    let num_items = config_data.number;
    let hidden = config_data.hidden_settings.is_some();
    let collection_in_cache = cache.items.get("-1").is_some();
    let mut item_redeemed = false;
    let mut freeze_deployed = false;

    let cache_items_sans_collection = (cache.items.len() - collection_in_cache as usize) as u64;

    if num_items != cache_items_sans_collection {
        return Err(anyhow!(
            "Number of items ({}) do not match cache items ({}). 
            Item number in the config should only include asset files, not the collection file.",
            num_items,
            cache_items_sans_collection
        ));
    } else {
        check_symbol(&config_data.symbol)?;
        check_seller_fee_basis_points(config_data.seller_fee_basis_points)?;
    }

    let total_steps = 2 + (collection_in_cache as u8) + (config_data.freeze_time.is_some() as u8)
        - (hidden as u8);

    let candy_pubkey = if candy_machine_address.is_empty() {
        println!(
            "{} {}Creating candy machine",
            style(format!("[1/{}]", total_steps)).bold().dim(),
            CANDY_EMOJI
        );
        info!("Candy machine address is empty, creating new candy machine...");

        let spinner = spinner_with_style();
        spinner.set_message("Creating candy machine...");

        let candy_keypair = Keypair::new();
        let candy_pubkey = candy_keypair.pubkey();

        let uuid = DEFAULT_UUID.to_string();
        let candy_data = create_candy_machine_data(&client, &config_data, uuid)?;
        let program = client.program(CANDY_MACHINE_ID);

        let treasury_wallet = match config_data.spl_token {
            Some(spl_token) => {
                if config_data.sol_treasury_account.is_some() {
                    return Err(anyhow!("If spl-token-account or spl-token is set then sol-treasury-account cannot be set"));
                }

                let token_account = config_data
                    .spl_token_account
                    .unwrap_or_else(|| get_associated_token_address(&program.payer(), &spl_token));

                // validates the mint address of the token accepted as payment
                check_spl_token(&program, &spl_token.to_string())?;

                // validates the spl token wallet to receive proceedings from SPL token payments
                check_spl_token_account(&program, &token_account.to_string())?;
                token_account
            }
            None => match config_data.sol_treasury_account {
                Some(sol_treasury_account) => sol_treasury_account,
                None => sugar_config.keypair.pubkey(),
            },
        };

        // all good, let's create the candy machine

        let sig = initialize_candy_machine(
            &config_data,
            &candy_keypair,
            candy_data,
            treasury_wallet,
            program,
        )?;
        info!("Candy machine initialized with sig: {}", sig);
        info!(
            "Candy machine created with address: {}",
            &candy_pubkey.to_string()
        );

        cache.program = CacheProgram::new_from_cm(&candy_pubkey);
        cache.sync_file()?;

        spinner.finish_and_clear();

        candy_pubkey
    } else {
        println!(
            "{} {}Loading candy machine",
            style(format!("[1/{}]", total_steps)).bold().dim(),
            CANDY_EMOJI
        );

        let candy_pubkey = match Pubkey::from_str(candy_machine_address) {
            Ok(pubkey) => pubkey,
            Err(_err) => {
                error!(
                    "Invalid candy machine address in cache file: {}!",
                    candy_machine_address
                );
                return Err(CacheError::InvalidCandyMachineAddress(
                    candy_machine_address.to_string(),
                )
                .into());
            }
        };

        match get_candy_machine_state(&Arc::clone(&sugar_config), &candy_pubkey) {
            Ok(candy_state) => {
                if candy_state.items_redeemed > 0 {
                    item_redeemed = true;
                }
                if is_feature_active(&candy_state.data.uuid, FREEZE_FEATURE_INDEX) {
                    freeze_deployed = true;
                }
            }
            Err(_) => {
                return Err(anyhow!("Candy machine from cache does't exist on chain!"));
            }
        }

        candy_pubkey
    };

    println!("{} {}", style("Candy machine ID:").bold(), candy_pubkey);

    if let Some(collection_item) = cache.items.get_mut("-1") {
        println!(
            "\n{} {}Creating and setting the collection NFT for candy machine",
            style(format!("[2/{}]", total_steps)).bold().dim(),
            COLLECTION_EMOJI
        );

        if item_redeemed {
            println!("\nAn item has already been minted and thus cannot modify the candy machine collection. Skipping...");
        } else if collection_item.on_chain {
            println!("\nCollection mint already deployed.");
        } else {
            let pb = spinner_with_style();
            pb.set_message("Sending create and set collection NFT transaction...");

            let (_, collection_mint) =
                create_and_set_collection(client, candy_pubkey, &mut cache, &config_data)?;

            pb.finish_and_clear();
            println!(
                "{} {}",
                style("Collection mint ID:").bold(),
                collection_mint
            );
        }
    }

    // Freeze settings check
    if let Some(freeze_time) = config_data.freeze_time {
        let step_num = 2 + (collection_in_cache as u8);
        println!(
            "\n{} {}Setting up candy machine with Freeze feature",
            style(format!("[{}/{}]", step_num, total_steps))
                .bold()
                .dim(),
            ICE_CUBE_EMOJI
        );

        if item_redeemed {
            println!("\nAn item has already been minted and thus the freeze feature cannot be set. Skipping...");
        } else if freeze_deployed {
            println!("Freeze feature already deployed, skipping...");
        } else {
            let pb = spinner_with_style();
            pb.set_message("Sending set freeze command...");
            let sig = enable_freeze(&program, &config_data, &candy_pubkey, freeze_time)?;

            pb.finish_and_clear();
            println!("{} {}", style("Tx signature:").bold(), sig);
        }
    }

    // Hidden Settings check needs to be the last action in this command, so we can update the hash with the final cache state.
    if !hidden {
        let step_num = 2 + (collection_in_cache as u8) + (config_data.freeze_time.is_some() as u8);
        println!(
            "\n{} {}Writing config lines",
            style(format!("[{}/{}]", step_num, total_steps))
                .bold()
                .dim(),
            PAPER_EMOJI
        );

        let config_lines = generate_config_lines(num_items, &cache.items)?;

        if config_lines.is_empty() {
            println!("\nAll config lines deployed.");
        } else {
            // clear the interruption handler value ahead of the upload
            args.interrupted.store(false, Ordering::SeqCst);

            let errors = upload_config_lines(
                Arc::clone(&sugar_config),
                candy_pubkey,
                &mut cache,
                config_lines,
                args.interrupted,
            )
            .await?;

            if !errors.is_empty() {
                let mut message = String::new();
                write!(
                    message,
                    "Failed to deploy all config lines, {0} error(s) occurred:",
                    errors.len()
                )?;

                let mut unique = HashSet::new();

                for err in errors {
                    unique.insert(err.to_string());
                }

                for u in unique {
                    message.push_str(&style("\n=> ").dim().to_string());
                    message.push_str(&u);
                }

                return Err(DeployError::AddConfigLineFailed(message).into());
            }
        }
    } else {
        // If hidden settings are enabled, update the hash value with the new cache file.
        println!("\nCandy machine with hidden settings deployed.");
        let hidden_settings = config_data.hidden_settings.as_ref().unwrap().clone();

        println!(
            "\nHidden settings hash: {}",
            hash_and_update(hidden_settings, &args.config, &mut config_data, &args.cache,)?
        );

        println!("\nUpdating candy machine state with new hash value:\n");
        let update_args = UpdateArgs {
            keypair: args.keypair,
            rpc_url: args.rpc_url,
            cache: args.cache,
            new_authority: None,
            config: args.config,
            candy_machine: Some(candy_pubkey.to_string()),
        };

        process_update(update_args)?;
    }

    Ok(())
}
