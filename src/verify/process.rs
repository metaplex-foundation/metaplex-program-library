use std::{thread, time::Duration};

use anchor_lang::AccountDeserialize;
use console::style;
use mpl_candy_machine::CandyMachine;

use crate::{
    cache::*,
    candy_machine::CANDY_MACHINE_ID,
    common::*,
    config::Cluster,
    constants::{CANDY_EMOJI, PAPER_EMOJI},
    pdas::get_collection_pda,
    utils::*,
    verify::VerifyError,
};

pub struct VerifyArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
}

#[derive(Debug)]
pub struct OnChainItem {
    pub name: String,
    pub uri: String,
}

pub fn process_verify(args: VerifyArgs) -> Result<()> {
    let sugar_config = sugar_setup(args.keypair, args.rpc_url)?;

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

    println!(
        "{} {}Loading candy machine",
        style("[1/2]").bold().dim(),
        CANDY_EMOJI
    );

    let pb = spinner_with_style();
    pb.set_message("Connecting...");

    let candy_machine_pubkey = match Pubkey::from_str(&cache.program.candy_machine) {
        Ok(pubkey) => pubkey,
        Err(_) => {
            pb.finish_and_clear();
            return Err(CacheError::InvalidCandyMachineAddress(
                cache.program.candy_machine.clone(),
            )
            .into());
        }
    };

    let client = setup_client(&sugar_config)?;
    let program = client.program(CANDY_MACHINE_ID);

    let data = match program.rpc().get_account_data(&candy_machine_pubkey) {
        Ok(account_data) => account_data,
        Err(err) => {
            return Err(VerifyError::FailedToGetAccountData(err.to_string()).into());
        }
    };
    let candy_machine: CandyMachine = CandyMachine::try_deserialize(&mut data.as_slice())?;
    let collection_info = get_collection_pda(&candy_machine_pubkey, &program).ok();

    pb.finish_with_message("Completed");

    println!(
        "\n{} {}Verification",
        style("[2/2]").bold().dim(),
        PAPER_EMOJI
    );

    if candy_machine.data.hidden_settings.is_none() {
        let num_items = candy_machine.data.items_available;
        let cache_items = &mut cache.items;
        let mut errors = Vec::new();

        println!("Verifying {} config line(s): (Ctrl+C to abort)", num_items);
        let pb = progress_bar_with_style(num_items as u64);
        // sleeps for a about 1 second
        let step: u64 = 1_000_000 / num_items as u64;

        for i in 0..num_items {
            let name_start = CONFIG_ARRAY_START
                + STRING_LEN_SIZE
                + CONFIG_LINE_SIZE * (i as usize)
                + CONFIG_NAME_OFFSET;
            let name_end = name_start + MAX_NAME_LENGTH;

            let uri_start = CONFIG_ARRAY_START
                + STRING_LEN_SIZE
                + CONFIG_LINE_SIZE * (i as usize)
                + CONFIG_URI_OFFSET;
            let uri_end = uri_start + MAX_URI_LENGTH;

            let name_error = format!("Failed to decode name for item {}", i);
            let name = String::from_utf8(data[name_start..name_end].to_vec())
                .expect(&name_error)
                .trim_matches(char::from(0))
                .to_string();

            let uri_error = format!("Failed to decode uri for item {}", i);
            let uri = String::from_utf8(data[uri_start..uri_end].to_vec())
                .expect(&uri_error)
                .trim_matches(char::from(0))
                .to_string();

            let on_chain_item = OnChainItem { name, uri };
            let cache_item = cache_items
                .get_mut(&i.to_string())
                .expect("Failed to get item from config.");

            if let Err(err) = items_match(cache_item, &on_chain_item) {
                cache_item.on_chain = false;
                errors.push((i.to_string(), err.to_string()));
            }

            pb.inc(1);
            thread::sleep(Duration::from_micros(step));
        }

        if !errors.is_empty() {
            pb.abandon_with_message(format!("{}", style("Verification failed ").red().bold()));
            cache.sync_file()?;

            let total = errors.len();
            println!("\nInvalid items found: ");

            for e in errors {
                println!("- Item {}: {}", e.0, e.1);
            }
            println!("\nCache updated - re-run `deploy`.");
            return Err(anyhow!("{} invalid item(s) found.", total));
        } else {
            pb.finish_with_message(format!(
                "{}",
                style("Config line verification successful ").green().bold()
            ));
        }
    } else {
        // nothing else to do, there are no config lines in a candy machine
        // with hidden settings
        println!("\nHidden settings enabled. No config items to verify.");
    }
    if candy_machine.items_redeemed > 0 {
        println!(
            "\nAn item has already been minted. Skipping candy machine collection verification..."
        );
    } else {
        let collection_mint_cache = cache.program.collection_mint.clone();
        let collection_needs_deploy = if let Some(collection_item) = cache.items.get("-1") {
            !collection_item.on_chain
        } else {
            false
        };
        let collection_item = cache.items.get_mut("-1");

        if let Some((_, collection_pda_account)) = collection_info {
            if collection_pda_account.mint.to_string() != collection_mint_cache {
                println!("\nInvalid collection state found");
                cache.program.collection_mint = collection_pda_account.mint.to_string();
                if let Some(collection_item) = collection_item {
                    collection_item.on_chain = false;
                }
                cache.sync_file()?;
                println!("Cache updated - re-run `deploy`.");
                return Err(anyhow!(
                    "Collection mint in cache {} doesn't match on chain collection mint {}!",
                    collection_mint_cache,
                    collection_pda_account.mint.to_string()
                ));
            } else if collection_needs_deploy {
                println!("\nInvalid collection state found - re-run `deploy`.");
                return Err(CacheError::InvalidState.into());
            }
        } else {
            let mut error_found = false;
            if collection_mint_cache != String::new() {
                error_found = true;
                cache.program.collection_mint = String::new();
            }
            if let Some(collection_item) = collection_item {
                error_found = true;
                collection_item.on_chain = false;
            }
            if error_found {
                cache.sync_file()?;
                println!("\nInvalid collection state found - re-run `deploy`.");
                return Err(CacheError::InvalidState.into());
            }
        }
    }

    let cluster = match get_cluster(program.rpc())? {
        Cluster::Devnet => "devnet",
        Cluster::Mainnet => "mainnet",
    };

    println!(
        "\nVerification successful. You're good to go!\n\nSee your candy machine at:\n  -> https://www.solaneyes.com/address/{}?cluster={}",
        cache.program.candy_machine,
        cluster
    );
    Ok(())
}

fn items_match(cache_item: &CacheItem, on_chain_item: &OnChainItem) -> Result<()> {
    if cache_item.name != on_chain_item.name {
        return Err(VerifyError::Mismatch(
            "name".to_string(),
            cache_item.name.clone(),
            on_chain_item.name.clone(),
        )
        .into());
    } else if cache_item.metadata_link != on_chain_item.uri {
        return Err(VerifyError::Mismatch(
            "uri".to_string(),
            cache_item.metadata_link.clone(),
            on_chain_item.uri.clone(),
        )
        .into());
    }

    Ok(())
}
