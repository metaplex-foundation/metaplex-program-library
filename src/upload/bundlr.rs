pub use anchor_client::solana_sdk::native_token::LAMPORTS_PER_SOL;
use async_trait::async_trait;
use bundlr_sdk::{tags::Tag, Bundlr, BundlrTx, SolanaSigner};
use clap::crate_version;
use console::style;
use futures::future::select_all;
use std::{cmp, collections::HashSet, ffi::OsStr, fs, path::Path, sync::Arc};
use tokio::time::{sleep, Duration};

use crate::{common::*, config::*, upload::*, utils::*};

/// The number os retries to fetch the Bundlr balance (MAX_RETRY * 5 seconds limit)
const MAX_RETRY: u64 = 15;

/// Time (ms) to wait until next try
const DELAY_UNTIL_RETRY: u64 = 5000;

pub struct BundlrHandler {
    client: Arc<Bundlr<SolanaSigner>>,
    pubkey: Pubkey,
    node: String,
}

impl BundlrHandler {
    /// Initialize a new BundlrHandler.
    pub async fn initialize(
        config_data: &ConfigData,
        sugar_config: &SugarConfig,
    ) -> Result<BundlrHandler> {
        let pb = spinner_with_style();
        pb.set_message("Connecting...");

        let pid = CANDY_MACHINE_V2.parse().expect("Failed to parse PID");
        let client = setup_client(sugar_config)?;
        let program = client.program(pid);
        let solana_cluster: Cluster = get_cluster(program.rpc())?;

        let bundlr_node = match config_data.upload_method {
            UploadMethod::Bundlr => match solana_cluster {
                Cluster::Devnet => BUNDLR_DEVNET,
                Cluster::Mainnet => BUNDLR_MAINNET,
            },
            _ => {
                return Err(anyhow!(format!(
                    "Upload method '{}' currently unsupported!",
                    &config_data.upload_method.to_string()
                )))
            }
        };

        let http_client = reqwest::Client::new();
        let bundlr_address =
            BundlrHandler::get_bundlr_solana_address(&http_client, bundlr_node).await?;

        let bundlr_pubkey = Pubkey::from_str(&bundlr_address)?;
        // get keypair as base58 string for Bundlr
        let keypair = bs58::encode(sugar_config.keypair.to_bytes()).into_string();
        let signer = SolanaSigner::from_base58(&keypair);

        let bundlr_client = Bundlr::new(
            bundlr_node.to_string(),
            "solana".to_string(),
            "sol".to_string(),
            signer,
        );

        pb.finish_with_message("Connected");

        Ok(BundlrHandler {
            client: Arc::new(bundlr_client),
            pubkey: bundlr_pubkey,
            node: bundlr_node.to_string(),
        })
    }

    /// Return the solana address for Bundlr.
    pub async fn get_bundlr_solana_address(http_client: &HttpClient, node: &str) -> Result<String> {
        let url = format!("{}/info", node);
        let data = http_client.get(&url).send().await?.json::<Value>().await?;
        let addresses = data.get("addresses").unwrap();
        let solana_address = addresses
            .get("solana")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        Ok(solana_address)
    }

    /// Add fund to the Bundlr address.
    pub async fn fund_bundlr_address(
        program: &Program,
        http_client: &HttpClient,
        bundlr_address: &Pubkey,
        node: &str,
        payer: &Keypair,
        amount: u64,
    ) -> Result<Response> {
        let ix = system_instruction::transfer(&payer.pubkey(), bundlr_address, amount);
        let recent_blockhash = program.rpc().get_latest_blockhash()?;
        let payer_pubkey = payer.pubkey();

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer_pubkey),
            &[payer],
            recent_blockhash,
        );

        println!("Funding address:");
        println!("  -> pubkey: {}", payer_pubkey);
        println!(
            "  -> lamports: {} (𑗏 {})",
            amount,
            amount as f64 / LAMPORTS_PER_SOL as f64
        );

        let sig = program
            .rpc()
            .send_and_confirm_transaction_with_spinner_and_commitment(
                &tx,
                CommitmentConfig::confirmed(),
            )?;

        println!("{} {sig}", style("Signature:").bold());

        let mut map = HashMap::new();
        map.insert("tx_id", sig.to_string());
        let url = format!("{}/account/balance/solana", node);
        let response = http_client.post(&url).json(&map).send().await?;

        Ok(response)
    }

    /// Return the Bundlr balance.
    pub async fn get_bundlr_balance(
        http_client: &HttpClient,
        address: &str,
        node: &str,
    ) -> Result<u64> {
        debug!("Getting balance for address: {address}");
        let url = format!("{}/account/balance/solana/?address={}", node, address);
        let response = http_client.get(&url).send().await?.json::<Value>().await?;
        let value = response.get("balance").unwrap();

        Ok(value.as_str().unwrap().parse::<u64>().unwrap())
    }

    /// Return the Bundlr fee for upload based on the data size.
    pub async fn get_bundlr_fee(
        http_client: &HttpClient,
        node: &str,
        data_size: u64,
    ) -> Result<u64> {
        let required_amount = http_client
            .get(format!("{node}/price/solana/{data_size}"))
            .send()
            .await?
            .text()
            .await?
            .parse::<u64>()?;
        Ok(required_amount)
    }

    /// Send a transaction to Bundlr and wait for a response.
    async fn send_bundlr_tx(
        bundlr_client: Arc<Bundlr<SolanaSigner>>,
        asset_id: String,
        tx: BundlrTx,
    ) -> Result<(String, String)> {
        let response = bundlr_client.send_transaction(tx).await?;
        let id = response.get("id").unwrap().as_str().unwrap();
        Ok((asset_id, id.to_string()))
    }
}

#[async_trait]
impl UploadHandler for BundlrHandler {
    /// Upload the data to Bundlr.
    async fn upload_data(
        &self,
        sugar_config: &SugarConfig,
        assets: &HashMap<usize, AssetPair>,
        cache: &mut Cache,
        indices: &[usize],
        data_type: DataType,
    ) -> Result<Vec<UploadError>> {
        // calculates the size of the files to upload
        let mut total_size = 0;
        let mut extension = HashSet::with_capacity(1);
        let mut paths = Vec::new();

        for index in indices {
            let item = assets.get(index).unwrap();
            // chooses the file path based on the data type
            let file_path = match data_type {
                DataType::Media => item.media.clone(),
                DataType::Metadata => item.metadata.clone(),
            };

            let path = Path::new(&file_path);
            total_size += 2000
                + cmp::max(
                    10000,
                    match data_type {
                        DataType::Media => std::fs::metadata(path)?.len(),
                        DataType::Metadata => {
                            let cache_item = cache.items.0.get(&index.to_string()).unwrap();
                            get_updated_metadata(item, cache_item)
                                .unwrap()
                                .into_bytes()
                                .len() as u64
                        }
                    },
                );

            let ext = path.extension().and_then(OsStr::to_str).unwrap();
            extension.insert(String::from(ext));

            paths.push(file_path);
        }

        // validates that all files have the same extension
        let extension = if extension.len() == 1 {
            extension.iter().next().unwrap()
        } else {
            return Err(anyhow!("Invalid file extension: {:?}", extension));
        };

        info!("Total upload size: {}", total_size);

        let http_client = reqwest::Client::new();

        let lamports_fee =
            BundlrHandler::get_bundlr_fee(&http_client, &self.node, total_size).await?;
        let address = sugar_config.keypair.pubkey().to_string();
        let mut balance =
            BundlrHandler::get_bundlr_balance(&http_client, &address, &self.node).await?;

        info!(
            "Bundlr balance {} lamports, require {} lamports",
            balance, lamports_fee
        );

        // funds the bundlr wallet for media upload

        let pid = CANDY_MACHINE_V2.parse().expect("Failed to parse PID");
        let client = setup_client(sugar_config)?;
        let program = client.program(pid);

        if lamports_fee > balance {
            BundlrHandler::fund_bundlr_address(
                &program,
                &http_client,
                &self.pubkey,
                &self.node,
                &sugar_config.keypair,
                lamports_fee - balance,
            )
            .await?;

            let pb = ProgressBar::new(MAX_RETRY);
            pb.set_style(ProgressStyle::default_bar().template("{spinner} {msg} {wide_bar}"));
            pb.enable_steady_tick(60);
            pb.set_message("Verifying balance:");

            // waits until the balance can be verified, otherwise the upload
            // will fail
            for _i in 0..MAX_RETRY {
                let res =
                    BundlrHandler::get_bundlr_balance(&http_client, &address, &self.node).await;

                if let Ok(value) = res {
                    balance = value;
                }

                if balance >= lamports_fee {
                    break;
                }

                sleep(Duration::from_millis(DELAY_UNTIL_RETRY)).await;
                pb.inc(1);
            }

            pb.finish_and_clear();

            if balance < lamports_fee {
                let error = UploadError::NoBundlrBalance(address).into();
                error!("{error}");
                return Err(error);
            }
        }

        let sugar_tag = Tag::new("App-Name".into(), format!("Sugar {}", crate_version!()));

        let media_tag = match data_type {
            DataType::Media => Tag::new("Content-Type".into(), format!("image/{extension}")),
            DataType::Metadata => Tag::new("Content-Type".into(), "application/json".to_string()),
        };

        // upload data to bundlr

        println!("\nSending data: (Ctrl+C to abort)");

        let pb = progress_bar_with_style(paths.len() as u64);
        let mut handles = Vec::new();

        for file_path in paths {
            let path = Path::new(&file_path);
            let asset_id = String::from(path.file_stem().and_then(OsStr::to_str).unwrap());

            let data = match data_type {
                DataType::Media => fs::read(&path)?,
                DataType::Metadata => {
                    let index = asset_id.parse::<usize>().unwrap();
                    let asset_pair = assets.get(&index).unwrap();
                    let cache_item = cache.items.0.get(&asset_id).unwrap();
                    // replaces the media link without modifying the original file to avoid
                    // changing the hash of the metadata file
                    get_updated_metadata(asset_pair, cache_item)
                        .unwrap()
                        .into_bytes()
                }
            };

            let bundlr_client = self.client.clone();
            let tx = bundlr_client
                .create_transaction_with_tags(data, vec![sugar_tag.clone(), media_tag.clone()]);

            let handle = tokio::spawn(async move {
                BundlrHandler::send_bundlr_tx(bundlr_client, asset_id.to_string(), tx).await
            });

            handles.push(handle);
        }

        let mut errors = Vec::new();

        while !handles.is_empty() {
            match select_all(handles).await {
                (Ok(res), _index, remaining) => {
                    // independently if the upload was successful or not
                    // we continue to try the remaining ones
                    handles = remaining;

                    if res.is_ok() {
                        let val = res?;
                        let link = format!("https://arweave.net/{}", val.clone().1);
                        // cache item to update
                        let item = cache.items.0.get_mut(&val.0).unwrap();

                        match data_type {
                            DataType::Media => item.media_link = link,
                            DataType::Metadata => item.metadata_link = link,
                        }
                        // saves the progress to the cache file
                        cache.sync_file()?;
                        // updates the progress bar
                        pb.inc(1);
                    } else {
                        // user will need to retry the upload
                        errors.push(UploadError::SendDataFailed(format!(
                            "Bundlr upload error: {:?}",
                            res.err().unwrap()
                        )));
                    }
                }
                (Err(err), _index, remaining) => {
                    errors.push(UploadError::SendDataFailed(format!(
                        "Bundlr upload error: {:?}",
                        err
                    )));
                    // ignoring all errors
                    handles = remaining;
                }
            }
        }

        if !errors.is_empty() {
            pb.abandon_with_message(format!("{}", style("Upload failed ").red().bold()));
        } else {
            pb.finish_with_message(format!("{}", style("Upload successful ").green().bold()));
        }

        Ok(errors)
    }
}
