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
use mpl_token_metadata::state::{Metadata, TokenMetadataAccount};

use crate::{
    cache::*,
    candy_machine::{get_candy_machine_state, CANDY_MACHINE_ID},
    common::*,
    config::parser::get_config_data,
    deploy::{
        create_candy_machine_data, create_collection, errors::*, generate_config_lines,
        initialize_candy_machine, upload_config_lines,
    },
    hash::hash_and_update,
    pdas::find_metadata_pda,
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
    let mut config_data = get_config_data(&args.config)?;

    let candy_machine_address = cache.program.candy_machine.clone();

    // checks the candy machine data

    let num_items = config_data.number;
    let hidden = config_data.hidden_settings.is_some();
    let collection_in_cache = cache.items.get("-1").is_some();

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

    let total_steps = 2 + if candy_machine_address.is_empty() {
        collection_in_cache as u8
    } else {
        0
    } - (hidden as u8);

    let candy_pubkey = if candy_machine_address.is_empty() {
        let candy_keypair = Keypair::new();
        let candy_pubkey = candy_keypair.pubkey();

        let collection_item = if let Some(collection_item) = cache.items.get_mut("-1") {
            collection_item
        } else {
            return Err(anyhow!("Missing collection item in cache"));
        };

        println!(
            "\n{} {}Creating collection NFT for candy machine",
            style(format!("[1/{}]", total_steps)).bold().dim(),
            COLLECTION_EMOJI
        );

        let collection_mint = if collection_item.on_chain {
            println!("\nCollection mint already deployed.");
            Pubkey::from_str(&cache.program.collection_mint)?
        } else {
            let pb = spinner_with_style();
            pb.set_message("Creating NFT...");

            let (_, collection_mint) =
                create_collection(&client, candy_pubkey, &mut cache, &config_data)?;

            pb.finish_and_clear();
            println!(
                "{} {}",
                style("Collection mint ID:").bold(),
                collection_mint
            );

            collection_mint
        };

        println!(
            "{} {}Creating candy machine",
            style(format!("\n[2/{}]", total_steps)).bold().dim(),
            CANDY_EMOJI
        );
        info!("Candy machine address is empty, creating new candy machine...");

        let spinner = spinner_with_style();
        spinner.set_message("Creating candy machine...");

        let candy_data = create_candy_machine_data(&client, &config_data, &cache)?;
        let program = client.program(CANDY_MACHINE_ID);

        // all good, let's create the candy machine

        let collection_metadata = find_metadata_pda(&collection_mint);
        let data = program.rpc().get_account_data(&collection_metadata)?;
        let metadata = Metadata::safe_deserialize(data.as_slice())?;

        let sig = initialize_candy_machine(
            &config_data,
            &candy_keypair,
            candy_data,
            collection_mint,
            metadata.update_authority,
            program,
        )?;
        info!("Candy machine initialized with sig: {}", sig);
        info!(
            "Candy machine created with address: {}",
            &candy_pubkey.to_string()
        );

        cache.program = CacheProgram::new_from_cm(&candy_pubkey);
        cache.program.collection_mint = collection_mint.to_string();
        cache.sync_file()?;

        spinner.finish_and_clear();

        candy_pubkey
    } else {
        println!(
            "{} {}Loading candy machine",
            style(format!("[1/{}]", total_steps)).bold().dim(),
            CANDY_EMOJI
        );

        let candy_pubkey = match Pubkey::from_str(&candy_machine_address) {
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

        if get_candy_machine_state(&Arc::clone(&sugar_config), &candy_pubkey).is_err() {
            println!(
                "\n{} Candy machine {} not found on-chain",
                WARNING_EMOJI, candy_machine_address
            );
            println!(
                "\nThis can happen if you are trying to re-deploy a candy machine from \
                    a previously used cache file. If this is the case, re-run the deploy command \
                    with the option '--new'.",
            );

            return Err(anyhow!(
                "Candy machine from cache does't exist on chain: {}",
                candy_machine_address
            ));
        }

        candy_pubkey
    };

    println!("{} {}", style("Candy machine ID:").bold(), candy_pubkey);

    // Hidden Settings check needs to be the last action in this command, so we can
    // update the hash with the final cache state.
    if !hidden {
        let step_num = 2 + if candy_machine_address.is_empty() {
            collection_in_cache as u8
        } else {
            0
        };
        println!(
            "\n{} {}Writing config lines",
            style(format!("[{}/{}]", step_num, total_steps))
                .bold()
                .dim(),
            PAPER_EMOJI
        );

        let cndy_state = get_candy_machine_state(&sugar_config, &candy_pubkey)?;
        let cndy_data = cndy_state.data;

        let config_lines = generate_config_lines(num_items, &cache.items, &cndy_data)?;

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
