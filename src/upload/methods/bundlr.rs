use std::{cmp, fs, path::Path, sync::Arc};

use anchor_client::solana_sdk::native_token::LAMPORTS_PER_SOL;
use async_trait::async_trait;
use bundlr_sdk::{tags::Tag, Bundlr, Ed25519Signer as SolanaSigner};
use clap::crate_version;
use console::style;
use solana_client::rpc_client::RpcClient;
use tokio::{
    task::JoinHandle,
    time::{sleep, Duration},
};

use crate::{
    candy_machine::CANDY_MACHINE_ID,
    common::*,
    config::*,
    upload::{
        assets::{get_updated_metadata, AssetPair, DataType},
        uploader::{AssetInfo, ParallelUploader, Prepare, MOCK_URI_SIZE},
    },
    utils::*,
};

/// The number os retries to fetch the Bundlr balance (MAX_RETRY * DELAY_UNTIL_RETRY ms limit)
const MAX_RETRY: u64 = 120;

/// Time (ms) to wait until next try
const DELAY_UNTIL_RETRY: u64 = 1000;

/// Size of Bundlr transaction header
const HEADER_SIZE: u64 = 2000;

/// Minimum file size for cost calculation
const MINIMUM_SIZE: u64 = 10000;

pub struct BundlrMethod {
    pub client: Arc<Bundlr<SolanaSigner>>,
    pub sugar_tag: Tag,
    pubkey: Pubkey,
    node: String,
}

impl BundlrMethod {
    pub async fn new(sugar_config: &SugarConfig, config_data: &ConfigData) -> Result<Self> {
        let client = setup_client(sugar_config)?;
        let program = client.program(CANDY_MACHINE_ID);
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
            BundlrMethod::get_bundlr_solana_address(&http_client, bundlr_node).await?;

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

        let sugar_tag = Tag::new("App-Name".into(), format!("Sugar {}", crate_version!()));

        Ok(Self {
            client: Arc::new(bundlr_client),
            pubkey: bundlr_pubkey,
            sugar_tag,
            node: bundlr_node.to_string(),
        })
    }

    /// Return the solana address for Bundlr.
    async fn get_bundlr_solana_address(http_client: &HttpClient, node: &str) -> Result<String> {
        let url = format!("{}/info", node);
        let data = http_client.get(&url).send().await?.json::<Value>().await?;
        let addresses = data
            .get("addresses")
            .expect("Failed to get bundlr addresses.");

        let solana_address = addresses
            .get("solana")
            .expect("Failed to get Solana address from bundlr.")
            .as_str()
            .expect("Solana bundlr address is not of type string.")
            .to_string();
        Ok(solana_address)
    }

    /// Add fund to the Bundlr address.
    async fn fund_bundlr_address(
        rpc_client: RpcClient,
        http_client: &HttpClient,
        bundlr_address: &Pubkey,
        node: &str,
        payer: &Keypair,
        amount: u64,
    ) -> Result<Response> {
        let ix = system_instruction::transfer(&payer.pubkey(), bundlr_address, amount);
        let recent_blockhash = rpc_client.get_latest_blockhash()?;
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
            "  -> lamports: {} (◎ {})",
            amount,
            amount as f64 / LAMPORTS_PER_SOL as f64
        );

        let sig = rpc_client.send_and_confirm_transaction_with_spinner_and_commitment(
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
        let value = response
            .get("balance")
            .expect("Failed to get balance from bundlr.");

        Ok(value
            .as_str()
            .unwrap()
            .parse::<u64>()
            .expect("Failed to parse bundlr balance."))
    }

    /// Return the Bundlr fee for upload based on the data size.
    async fn get_bundlr_fee(http_client: &HttpClient, node: &str, data_size: u64) -> Result<u64> {
        let required_amount = http_client
            .get(format!("{node}/price/solana/{data_size}"))
            .send()
            .await?
            .text()
            .await?
            .parse::<u64>()?;
        Ok(required_amount)
    }

    async fn send(
        client: Arc<Bundlr<SolanaSigner>>,
        tag: Tag,
        asset_info: AssetInfo,
    ) -> Result<(String, String)> {
        let data = match asset_info.data_type {
            DataType::Image => fs::read(&asset_info.content)?,
            DataType::Metadata => asset_info.content.into_bytes(),
            DataType::Animation => fs::read(&asset_info.content)?,
        };

        let tags = vec![
            tag,
            Tag::new("Content-Type".into(), asset_info.content_type.clone()),
        ];

        let tx = client.create_transaction_with_tags(data, tags);
        let response = client.send_transaction(tx).await?;
        let id = response
            .get("id")
            .expect("Failed to convert transaction id to string.")
            .as_str()
            .expect("Failed to get an id from bundlr transaction.");

        let link = format!("https://arweave.net/{}", id);

        Ok((asset_info.asset_id, link))
    }
}

#[async_trait]
impl Prepare for BundlrMethod {
    async fn prepare(
        &self,
        sugar_config: &SugarConfig,
        assets: &HashMap<isize, AssetPair>,
        asset_indices: Vec<(DataType, &[isize])>,
    ) -> Result<()> {
        // calculates the size of the files to upload
        let mut total_size = 0;

        for (data_type, indices) in asset_indices {
            match data_type {
                DataType::Image => {
                    for index in indices {
                        let item = assets.get(index).unwrap();
                        let path = Path::new(&item.image);
                        total_size +=
                            HEADER_SIZE + cmp::max(MINIMUM_SIZE, fs::metadata(path)?.len());
                    }
                }
                DataType::Animation => {
                    for index in indices {
                        let item = assets.get(index).unwrap();

                        if let Some(animation) = &item.animation {
                            let path = Path::new(animation);
                            total_size +=
                                HEADER_SIZE + cmp::max(MINIMUM_SIZE, fs::metadata(path)?.len());
                        }
                    }
                }
                DataType::Metadata => {
                    let mock_uri = "x".repeat(MOCK_URI_SIZE);

                    for index in indices {
                        let item = assets.get(index).unwrap();
                        let animation = if item.animation.is_some() {
                            Some(mock_uri.clone())
                        } else {
                            None
                        };

                        total_size += HEADER_SIZE
                            + cmp::max(
                                MINIMUM_SIZE,
                                get_updated_metadata(&item.metadata, &mock_uri.clone(), &animation)?
                                    .into_bytes()
                                    .len() as u64,
                            );
                    }
                }
            }
        }

        info!("Total upload size: {}", total_size);

        let http_client = reqwest::Client::new();

        let lamports_fee =
            BundlrMethod::get_bundlr_fee(&http_client, &self.node, total_size).await?;
        let address = sugar_config.keypair.pubkey().to_string();
        let mut balance =
            BundlrMethod::get_bundlr_balance(&http_client, &address, &self.node).await?;

        info!(
            "Bundlr balance {} lamports, require {} lamports",
            balance, lamports_fee
        );

        // funds the bundlr wallet for media upload

        let rpc_client = {
            let client = setup_client(sugar_config)?;
            let program = client.program(CANDY_MACHINE_ID);
            program.rpc()
        };

        if lamports_fee > balance {
            BundlrMethod::fund_bundlr_address(
                rpc_client,
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
                    BundlrMethod::get_bundlr_balance(&http_client, &address, &self.node).await;

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
                let error = anyhow!(format!(
                    "No Bundlr balance found for address: {0}, check \
                    Bundlr cluster and address balance",
                    address
                ));
                error!("{error}");
                return Err(error);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl ParallelUploader for BundlrMethod {
    fn upload_asset(&self, asset_info: AssetInfo) -> JoinHandle<Result<(String, String)>> {
        let client = self.client.clone();
        let tag = self.sugar_tag.clone();
        tokio::spawn(async move { BundlrMethod::send(client, tag, asset_info).await })
    }
}
