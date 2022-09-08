use std::sync::{Arc, Mutex};

use anchor_client::solana_sdk::account::Account;
use anchor_lang::AnchorDeserialize;
use console::style;
use futures::future::join_all;
use mpl_token_metadata::{
    instruction::update_metadata_accounts_v2,
    state::{DataV2, Metadata},
    ID as TOKEN_METADATA_PROGRAM_ID,
};
use serde::Serialize;
use solana_client::{client_error::ClientError, rpc_client::RpcClient};
use solana_transaction_crawler::crawler::Crawler;
use tokio::sync::Semaphore;

use crate::{
    cache::load_cache,
    candy_machine::CANDY_MACHINE_ID,
    common::*,
    config::{get_config_data, Cluster},
    pdas::{find_candy_machine_creator_pda, find_metadata_pda},
    setup::get_rpc_url,
    utils::*,
};

pub struct RevealArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub config: String,
}

#[derive(Clone, Debug)]
pub struct MetadataUpdateValues {
    pub metadata_pubkey: Pubkey,
    pub metadata: Metadata,
    pub new_uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RevealTx {
    metadata_pubkey: Pubkey,
    result: RevealResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
enum RevealResult {
    Success,
    Failure(String),
}

pub async fn process_reveal(args: RevealArgs) -> Result<()> {
    println!(
        "{} {}Loading items from the cache",
        style("[1/4]").bold().dim(),
        LOOKING_GLASS_EMOJI
    );

    let spinner = spinner_with_style();
    spinner.set_message("Connecting...");

    let config = get_config_data(&args.config)?;

    // If it's not a Hidden Settings mint, return an error.
    let _hidden_settings = if let Some(hidden_settings) = config.hidden_settings {
        hidden_settings
    } else {
        return Err(anyhow!("Candy machine is not a Hidden Settings mint."));
    };

    let cache = load_cache(&args.cache, false)?;
    let sugar_config = sugar_setup(args.keypair, args.rpc_url.clone())?;
    let anchor_client = setup_client(&sugar_config)?;
    let program = anchor_client.program(CANDY_MACHINE_ID);

    let candy_machine_id = match Pubkey::from_str(&cache.program.candy_machine) {
        Ok(candy_machine_id) => candy_machine_id,
        Err(_) => {
            let error = anyhow!(
                "Failed to parse candy machine id: {}",
                &cache.program.candy_machine
            );
            error!("{:?}", error);
            return Err(error);
        }
    };

    spinner.finish_with_message("Done");

    println!(
        "\n{} {}Getting minted NFTs for candy machine {}",
        style("[2/4]").bold().dim(),
        LOOKING_GLASS_EMOJI,
        candy_machine_id
    );

    let spinner = spinner_with_style();
    spinner.set_message("Loading...");
    let solana_cluster: Cluster = get_cluster(program.rpc())?;
    let rpc_url = get_rpc_url(args.rpc_url);

    let solana_cluster = if rpc_url.ends_with("8899") {
        Cluster::Localnet
    } else {
        solana_cluster
    };

    let metadata_pubkeys = match solana_cluster {
        Cluster::Devnet | Cluster::Localnet => {
            let client = RpcClient::new(&rpc_url);
            let (creator, _) = find_candy_machine_creator_pda(&candy_machine_id);
            let creator = bs58::encode(creator).into_string();
            get_cm_creator_accounts(&client, &creator, 0)?
        }
        Cluster::Mainnet => {
            let client = RpcClient::new(&rpc_url);
            let crawled_accounts = Crawler::get_cmv2_mints(client, candy_machine_id).await?;
            match crawled_accounts.get("metadata") {
                Some(accounts) => accounts
                    .iter()
                    .map(|account| Pubkey::from_str(account).unwrap())
                    .collect::<Vec<Pubkey>>(),
                None => Vec::new(),
            }
        }
        _ => {
            return Err(anyhow!(
                "Cluster being used is unsupported for this command."
            ))
        }
    };

    if metadata_pubkeys.is_empty() {
        spinner.finish_with_message(format!(
            "{}{:?}",
            style("No NFTs found on ").red().bold(),
            style(solana_cluster).red().bold()
        ));
        return Err(anyhow!(
            "No minted NFTs found for candy machine {}",
            candy_machine_id
        ));
    }

    spinner.finish_with_message(format!(
        "Found {:?} accounts",
        metadata_pubkeys.len() as u64
    ));

    println!(
        "\n{} {}Matching NFTs to cache values",
        style("[3/4]").bold().dim(),
        LOOKING_GLASS_EMOJI
    );
    let spinner = spinner_with_style();

    let mut futures = Vec::new();
    let client = RpcClient::new(&rpc_url);
    let client = Arc::new(client);

    // Get all metadata accounts.
    metadata_pubkeys.as_slice().chunks(100).for_each(|chunk| {
        let client = client.clone();
        futures.push(async move { async_get_multiple_accounts(client, chunk).await });
    });
    let results = join_all(futures).await;
    let mut accounts = Vec::new();

    for result in results {
        let res = result.unwrap();
        accounts.extend(res);
    }

    let metadata: Vec<Metadata> = accounts
        .into_iter()
        .map(|a| a.unwrap().data)
        .map(|d| Metadata::deserialize(&mut d.as_slice()).unwrap())
        .collect();

    // Convert cache to make keys match NFT numbers.
    let nft_lookup: HashMap<String, &CacheItem> = cache
        .items
        .iter()
        .filter(|(k, _)| *k != "-1") // skip collection index
        .map(|(k, item)| (increment_key(k), item))
        .collect();

    spinner.finish_with_message("Done");

    let mut update_values = Vec::new();

    println!(
        "\n{} {}Updating NFT URIs from cache values",
        style("[4/4]").bold().dim(),
        UPLOAD_EMOJI
    );

    let pattern = regex::Regex::new(r"#([0-9]+)").expect("Failed to create regex pattern.");

    let spinner = spinner_with_style();
    spinner.set_message("Setting up transactions...");
    for m in metadata {
        let name = m.data.name.trim_matches(char::from(0)).to_string();
        let capture = pattern
            .captures(&name)
            .map(|c| c[0].to_string())
            .ok_or_else(|| anyhow!("No captures found for {name}"))?;
        let num = capture
            .split('#')
            .nth(1)
            .ok_or_else(|| anyhow!("No NFT number found for name: {name}"))?;

        let metadata_pubkey = find_metadata_pda(&m.mint);
        let new_uri = nft_lookup
            .get(num)
            .ok_or_else(|| anyhow!("No URI found for number: {num}"))?
            .metadata_link
            .clone();
        update_values.push(MetadataUpdateValues {
            metadata_pubkey,
            metadata: m,
            new_uri,
        });
    }
    spinner.finish_and_clear();

    let keypair = Arc::new(sugar_config.keypair);
    let sem = Arc::new(Semaphore::new(1000));
    let reveal_results = Arc::new(Mutex::new(Vec::new()));
    let mut tx_tasks = Vec::new();

    let pb = progress_bar_with_style(metadata_pubkeys.len() as u64);
    pb.set_message("Updating NFTs... ");

    for item in update_values {
        let permit = Arc::clone(&sem).acquire_owned().await.unwrap();
        let client = client.clone();
        let keypair = keypair.clone();
        let reveal_results = reveal_results.clone();
        let pb = pb.clone();

        tx_tasks.push(tokio::spawn(async move {
            // Move permit into the closure so it is dropped when the task is dropped.
            let _permit = permit;
            let metadata_pubkey = item.metadata_pubkey;
            let mut tx = RevealTx {
                metadata_pubkey,
                result: RevealResult::Success,
            };

            match update_metadata_value(client, keypair, item).await {
                Ok(_) => reveal_results.lock().unwrap().push(tx),
                Err(e) => {
                    tx.result = RevealResult::Failure(e.to_string());
                    reveal_results.lock().unwrap().push(tx);
                }
            }

            pb.inc(1);
        }));
    }

    for task in tx_tasks {
        task.await.unwrap();
    }
    pb.finish();

    let results = reveal_results.lock().unwrap();

    let errors: Vec<&RevealTx> = results
        .iter()
        .filter(|r| matches!(r.result, RevealResult::Failure(_)))
        .collect();

    if !errors.is_empty() {
        println!(
            "{}Some reveals failed. See the reveal cache file for details. Re-run the command.",
            WARNING_EMOJI
        );
        let f = File::create("sugar-reveal-cache.json")
            .map_err(|e| anyhow!("Failed to create sugar reveal cache file: {e}"))?;
        serde_json::to_writer_pretty(f, &errors).unwrap();
    } else {
        println!("\n{}Reveal complete!", CONFETTI_EMOJI);
    }

    Ok(())
}

async fn async_get_multiple_accounts(
    client: Arc<RpcClient>,
    pubkeys: &[Pubkey],
) -> Result<Vec<Option<Account>>, ClientError> {
    client.get_multiple_accounts(pubkeys)
}

async fn update_metadata_value(
    client: Arc<RpcClient>,
    update_authority: Arc<Keypair>,
    value: MetadataUpdateValues,
) -> Result<(), ClientError> {
    let mut data = value.metadata.data;
    if data.uri.trim_matches(char::from(0)) != value.new_uri.trim_matches(char::from(0)) {
        data.uri = value.new_uri;

        let data_v2 = DataV2 {
            name: data.name,
            symbol: data.symbol,
            uri: data.uri,
            seller_fee_basis_points: data.seller_fee_basis_points,
            creators: data.creators,
            collection: value.metadata.collection,
            uses: value.metadata.uses,
        };

        let ix = update_metadata_accounts_v2(
            TOKEN_METADATA_PROGRAM_ID,
            value.metadata_pubkey,
            update_authority.pubkey(),
            None,
            Some(data_v2),
            None,
            None,
        );

        let recent_blockhash = client.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&update_authority.pubkey()),
            &[&*update_authority],
            recent_blockhash,
        );

        client.send_and_confirm_transaction(&tx)?;
    }

    Ok(())
}

fn increment_key(key: &str) -> String {
    (key.parse::<u32>()
        .expect("Key parsing out of bounds for u32.")
        + 1)
    .to_string()
}
