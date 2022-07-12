use std::{
    cmp,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anchor_client::solana_sdk::{pubkey::Pubkey, signature::Keypair};
use anyhow::Result;
use console::style;
use futures::future::select_all;
use mpl_candy_machine::{accounts as nft_accounts, instruction as nft_instruction, ConfigLine};
pub use mpl_token_metadata::state::{
    MAX_CREATOR_LIMIT, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH,
};

use crate::{
    cache::*, candy_machine::CANDY_MACHINE_ID, common::*, config::data::*, deploy::errors::*,
    setup::setup_client, utils::*,
};

/// The maximum config line bytes per transaction.
const MAX_TRANSACTION_BYTES: usize = 1000;

/// The maximum number of config lines per transaction.
const MAX_TRANSACTION_LINES: usize = 17;

pub struct TxInfo {
    candy_pubkey: Pubkey,
    payer: Keypair,
    chunk: Vec<(u32, ConfigLine)>,
}

/// Determine the config lines that need to be uploaded.
pub fn generate_config_lines(
    num_items: u64,
    cache_items: &CacheItems,
) -> Result<Vec<Vec<(u32, ConfigLine)>>> {
    let mut config_lines: Vec<Vec<(u32, ConfigLine)>> = Vec::new();
    let mut current: Vec<(u32, ConfigLine)> = Vec::new();
    let mut tx_size = 0;

    for i in 0..num_items {
        let item = match cache_items.get(&i.to_string()) {
            Some(item) => item,
            None => {
                return Err(
                    DeployError::AddConfigLineFailed(format!("Missing cache item {}", i)).into(),
                );
            }
        };

        if item.on_chain {
            // if the current item is on-chain already, store the previous
            // items as a transaction since we cannot have gaps in the indices
            // to write the config lines
            if !current.is_empty() {
                config_lines.push(current);
                current = Vec::new();
                tx_size = 0;
            }
        } else {
            let config_line = item
                .to_config_line()
                .expect("Could not convert item to config line");

            let size = (2 * STRING_LEN_SIZE) + config_line.name.len() + config_line.uri.len();

            if (tx_size + size) > MAX_TRANSACTION_BYTES || current.len() == MAX_TRANSACTION_LINES {
                // we need a separate tx to not break the size limit
                config_lines.push(current);
                current = Vec::new();
                tx_size = 0;
            }

            tx_size += size;
            current.push((i as u32, config_line));
        }
    }
    // adds the last chunk (if there is one)
    if !current.is_empty() {
        config_lines.push(current);
    }

    Ok(config_lines)
}

/// Send the config lines to the candy machine program.
pub async fn upload_config_lines(
    sugar_config: Arc<SugarConfig>,
    candy_pubkey: Pubkey,
    cache: &mut Cache,
    config_lines: Vec<Vec<(u32, ConfigLine)>>,
    interrupted: Arc<AtomicBool>,
) -> Result<Vec<DeployError>> {
    println!(
        "Sending config line(s) in {} transaction(s): (Ctrl+C to abort)",
        config_lines.len()
    );

    let pb = progress_bar_with_style(config_lines.len() as u64);

    debug!("Num of config line chunks: {:?}", config_lines.len());
    info!("Uploading config lines in chunks...");

    let mut transactions = Vec::new();

    for chunk in config_lines {
        let keypair = bs58::encode(sugar_config.keypair.to_bytes()).into_string();
        let payer = Keypair::from_base58_string(&keypair);

        transactions.push(TxInfo {
            candy_pubkey,
            payer,
            chunk,
        });
    }

    let mut handles = Vec::new();

    for tx in transactions.drain(0..cmp::min(transactions.len(), PARALLEL_LIMIT)) {
        let config = sugar_config.clone();
        handles.push(tokio::spawn(
            async move { add_config_lines(config, tx).await },
        ));
    }

    let mut errors = Vec::new();

    while !interrupted.load(Ordering::SeqCst) && !handles.is_empty() {
        match select_all(handles).await {
            (Ok(res), _index, remaining) => {
                // independently if the upload was successful or not
                // we continue to try the remaining ones
                handles = remaining;

                if res.is_ok() {
                    let indices = res?;

                    for index in indices {
                        let item = cache.items.get_mut(&index.to_string()).unwrap();
                        item.on_chain = true;
                    }
                    // updates the progress bar
                    pb.inc(1);
                } else {
                    // user will need to retry the upload
                    errors.push(DeployError::AddConfigLineFailed(format!(
                        "Transaction error: {:?}",
                        res.err().unwrap()
                    )));
                }
            }
            (Err(err), _index, remaining) => {
                // user will need to retry the upload
                errors.push(DeployError::AddConfigLineFailed(format!(
                    "Transaction error: {:?}",
                    err
                )));
                // ignoring all errors
                handles = remaining;
            }
        }

        if !transactions.is_empty() {
            // if we are half way through, let spawn more transactions
            if (PARALLEL_LIMIT - handles.len()) > (PARALLEL_LIMIT / 2) {
                // saves the progress to the cache file
                cache.sync_file()?;

                for tx in transactions.drain(0..cmp::min(transactions.len(), PARALLEL_LIMIT / 2)) {
                    let config = sugar_config.clone();
                    handles.push(tokio::spawn(
                        async move { add_config_lines(config, tx).await },
                    ));
                }
            }
        }
    }

    if !errors.is_empty() {
        pb.abandon_with_message(format!("{}", style("Deploy failed ").red().bold()));
    } else if !transactions.is_empty() {
        pb.abandon_with_message(format!("{}", style("Upload aborted ").red().bold()));
        return Err(DeployError::AddConfigLineFailed(
            "Not all config lines were deployed.".to_string(),
        )
        .into());
    } else {
        pb.finish_with_message(format!(
            "{}",
            style("Write config lines successful ").green().bold()
        ));
    }

    // makes sure the cache file is updated
    cache.sync_file()?;

    Ok(errors)
}

/// Send the `add_config_lines` instruction to the candy machine program.
pub async fn add_config_lines(config: Arc<SugarConfig>, tx_info: TxInfo) -> Result<Vec<u32>> {
    let client = setup_client(&config)?;
    let program = client.program(CANDY_MACHINE_ID);

    // this will be used to update the cache
    let mut indices: Vec<u32> = Vec::new();
    // configLine does not implement clone, so we have to do this
    let mut config_lines: Vec<ConfigLine> = Vec::new();
    // start index
    let start_index = tx_info.chunk[0].0;

    for (index, line) in tx_info.chunk {
        indices.push(index);
        config_lines.push(line);
    }

    let _sig = program
        .request()
        .accounts(nft_accounts::AddConfigLines {
            candy_machine: tx_info.candy_pubkey,
            authority: program.payer(),
        })
        .args(nft_instruction::AddConfigLines {
            index: start_index,
            config_lines,
        })
        .signer(&tx_info.payer)
        .send()?;

    Ok(indices)
}
