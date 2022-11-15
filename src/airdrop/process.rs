use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};

use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use console::style;
use tokio::sync::Semaphore;

use crate::{
    airdrop::{
        errors::AirDropError,
        structs::{AirDropTargets, TransactionResult},
        utils::{load_airdrop_list, load_airdrop_results, write_airdrop_results},
    },
    cache::load_cache,
    candy_machine::{CANDY_MACHINE_ID, *},
    common::*,
    mint::mint,
    pdas::*,
    utils::*,
};

pub struct AirdropArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub candy_machine: Option<String>,
    pub airdrop_list: String,
}

pub async fn process_airdrop(args: AirdropArgs) -> Result<()> {
    let sugar_config = sugar_setup(args.keypair, args.rpc_url)?;
    let client = Arc::new(setup_client(&sugar_config)?);
    let mut airdrop_list: AirDropTargets = load_airdrop_list(args.airdrop_list)?;

    // load_airdrop_results syncs airdrop_list and airdrop_results in case of rerun failures
    let airdrop_total_original = airdrop_list.iter().fold(0, |acc, x| acc + x.1);
    let airdrop_results = Arc::new(Mutex::new(load_airdrop_results(&mut airdrop_list)?));
    let airdrop_total = airdrop_list.iter().fold(0, |acc, x| acc + x.1);

    if airdrop_total_original != airdrop_total {
        print!(
            "Skipping {} mints due to existing transactions in airdrop_results.json",
            airdrop_total_original - airdrop_total
        );
    }

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

    let candy_machine_state = Arc::new(get_candy_machine_state(&sugar_config, &candy_pubkey)?);

    let collection_pda_info =
        Arc::new(get_collection_pda(&candy_pubkey, &client.program(CANDY_MACHINE_ID)).ok());

    pb.finish_with_message("Done");

    println!(
        "\n{} {}Minting from candy machine",
        style("[2/2]").bold().dim(),
        CANDY_EMOJI
    );

    let available = candy_machine_state.data.items_available - candy_machine_state.items_redeemed;

    if airdrop_total > available {
        return Err(
            AirDropError::AirdropTotalIsHigherThanAvailable(airdrop_total, available).into(),
        );
    }

    info!("Minting NFT from candy machine: {}", &candy_machine_id);
    info!("Candy machine program id: {:?}", CANDY_MACHINE_ID);

    let pb = progress_bar_with_style(airdrop_total);
    let mut tasks = Vec::new();
    let semaphore = Arc::new(Semaphore::new(10));
    let config = Arc::new(sugar_config);

    for (address, num) in airdrop_list.drain() {
        for _i in 0..num {
            let results = airdrop_results.clone();
            let config = config.clone();
            let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();
            let candy_machine_state = candy_machine_state.clone();
            let collection_pda_info = collection_pda_info.clone();
            let target = address.0;
            let pb = pb.clone();

            // Start tasks
            tasks.push(tokio::spawn(async move {
                let _permit = permit;
                let res = mint(
                    config,
                    candy_pubkey,
                    candy_machine_state,
                    collection_pda_info,
                    target,
                )
                .await;
                pb.inc(1);

                let mut results = results.lock().unwrap();
                results.entry(address).or_insert_with(Vec::new);
                let signatures = results.get_mut(&address).unwrap();

                match &res {
                    Ok(signature) => {
                        signatures.push(TransactionResult {
                            signature: signature.to_string(),
                            status: true,
                        });
                    }
                    Err(err) => {
                        signatures.push(TransactionResult {
                            signature: err.to_string(),
                            status: false,
                        });
                    }
                }

                res
            }));
        }
    }

    let mut error_count = 0;

    // Resolve tasks
    for task in tasks {
        let res = task.await.unwrap();
        if let Err(e) = res {
            error_count += 1;
            error!("{:?}, continuing. . .", e);
        }
    }

    write_airdrop_results(&airdrop_results.lock().unwrap())?;
    if error_count > 0 {
        pb.abandon_with_message(format!(
            "{} {} items failed.",
            style("Some of the items failed to mint.").red().bold(),
            error_count
        ));
        return Err(anyhow!(
            "{} {}/{} {}",
            style("Minted").red().bold(),
            airdrop_total - error_count,
            airdrop_total,
            style("of the items").red().bold()
        ));
    }
    pb.finish();

    Ok(())
}
