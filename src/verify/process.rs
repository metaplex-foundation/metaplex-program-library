use anchor_lang::AccountDeserialize;
use console::style;
use std::{thread, time::Duration};

use mpl_candy_machine::CandyMachine;

use crate::cache::*;
use crate::common::*;
use crate::constants::{CANDY_EMOJI, PAPER_EMOJI};
use crate::utils::*;
use crate::verify::VerifyError;

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

    if cache.items.0.is_empty() {
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

    let candy_machine_pubkey = Pubkey::from_str(&cache.program.candy_machine)?;
    let pid = CANDY_MACHINE_V2.parse().expect("Failed to parse PID");
    let client = setup_client(&sugar_config)?;
    let program = client.program(pid);

    let data = match program.rpc().get_account_data(&candy_machine_pubkey) {
        Ok(account_data) => account_data,
        Err(err) => {
            return Err(VerifyError::FailedToGetAccountData(err.to_string()).into());
        }
    };
    let candy_machine: CandyMachine = CandyMachine::try_deserialize(&mut data.as_slice())?;

    pb.finish_with_message("Completed");

    println!(
        "\n{} {}Verification",
        style("[2/2]").bold().dim(),
        PAPER_EMOJI
    );

    if candy_machine.data.hidden_settings.is_none() {
        let num_items = cache.items.0.len();
        let cache_items = &mut cache.items.0;
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

        pb.finish();

        if !errors.is_empty() {
            cache.sync_file()?;

            let total = errors.len();
            println!("\nInvalid items found: ");

            for e in errors {
                println!("- Item {}: {}", e.0, e.1);
            }
            println!("\nCache updated - re-run `deploy`.");

            return Err(anyhow!("{} invalid item(s) found.", total));
        }

        println!("\nAll items checked out. You're good to go!");
    } else {
        // nothing else todo, there are no config lines in a candy machine
        // with hidden settings
        println!("\nHidden settings enabled. You're good to go!");
    }

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
