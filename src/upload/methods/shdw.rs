use std::{fs, ops::Deref, sync::Arc};

use async_trait::async_trait;
use data_encoding::HEXLOWER;
use reqwest::{
    multipart::{Form, Part},
    StatusCode,
};
use ring::digest::{Context, SHA256};
use solana_program::pubkey;
use tokio::task::JoinHandle;

use crate::{
    common::*,
    config::*,
    upload::{
        assets::{get_updated_metadata, AssetPair, DataType},
        uploader::{AssetInfo, ParallelUploader, Prepare, MOCK_URI_SIZE},
        UploadError,
    },
    utils::*,
};

// Shadow Drive program id.
const SHADOW_DRIVE_PROGRAM_ID: Pubkey = pubkey!("2e1wdyNhUvE76y6yUCvah2KaviavMJYKoRun8acMRBZZ");
// Shadow Drive mainnet endpoint.
const MAINNET_ENDPOINT: &str = "https://shadow-storage.genesysgo.net";
// Shadow Drive devnet endpoint.
const DEVNET_ENDPOINT: &str = "https://shadow-storage-dev.genesysgo.net";
// Shadow Drive files location.
const SHDW_DRIVE_LOCATION: &str = "https://shdw-drive.genesysgo.net";

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct StorageInfo {
    pub reserved_bytes: u64,
    pub current_usage: u64,
    pub immutable: bool,
    pub owner1: Option<String>,
    pub owner2: Option<String>,
}

pub struct Config {
    endpoint: String,
    keypair: Keypair,
    storage_account: Pubkey,
    storage_info: StorageInfo,
}

pub struct SHDWMethod(Arc<Config>);

impl Deref for SHDWMethod {
    type Target = Arc<Config>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SHDWMethod {
    pub async fn new(sugar_config: &SugarConfig, config_data: &ConfigData) -> Result<Self> {
        if let Some(pubkey) = &config_data.shdw_storage_account {
            let client = setup_client(sugar_config)?;
            let program = client.program(SHADOW_DRIVE_PROGRAM_ID);
            let solana_cluster: Cluster = get_cluster(program.rpc())?;

            let endpoint = match solana_cluster {
                Cluster::Devnet => DEVNET_ENDPOINT,
                Cluster::Mainnet => MAINNET_ENDPOINT,
            };

            let http_client = reqwest::Client::new();
            let mut json = HashMap::new();
            json.insert("storage_account", pubkey);

            let response = http_client
                .post(format!("{endpoint}/storage-account-info"))
                .json(&json)
                .send()
                .await?;

            let key_bytes = sugar_config.keypair.to_bytes();
            let keypair = Keypair::from_bytes(&key_bytes)?;

            match response.status() {
                StatusCode::OK => {
                    let body = response.json::<Value>().await?;
                    let storage_info: StorageInfo = serde_json::from_value(body)?;

                    Ok(Self(Arc::new(Config {
                        endpoint: endpoint.to_string(),
                        keypair,
                        storage_account: Pubkey::from_str(pubkey)?,
                        storage_info,
                    })))
                }
                code => Err(anyhow!("Could not initialize storage account: {code}")),
            }
        } else {
            Err(anyhow!(
                "Missing 'shdwStorageAccount' value in config file."
            ))
        }
    }
}

#[async_trait]
impl Prepare for SHDWMethod {
    async fn prepare(
        &self,
        _sugar_config: &SugarConfig,
        assets: &HashMap<isize, AssetPair>,
        asset_indices: Vec<(DataType, &[isize])>,
    ) -> Result<()> {
        // calculates the size of the files to upload, this assumes that the total
        // storage has enough space to hold the collection as assets might already
        // exist and therefore will be replaced
        let mut total_size = 0;

        for (data_type, indices) in asset_indices {
            match data_type {
                DataType::Image => {
                    for index in indices {
                        let item = assets.get(index).unwrap();
                        let path = Path::new(&item.image);
                        total_size += fs::metadata(path)?.len();
                    }
                }
                DataType::Animation => {
                    for index in indices {
                        let item = assets.get(index).unwrap();

                        if let Some(animation) = &item.animation {
                            let path = Path::new(animation);
                            total_size += fs::metadata(path)?.len();
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

                        total_size +=
                            get_updated_metadata(&item.metadata, &mock_uri.clone(), &animation)?
                                .into_bytes()
                                .len() as u64;
                    }
                }
            }
        }

        if self.storage_info.reserved_bytes < total_size {
            let required = total_size - self.storage_info.reserved_bytes;
            return Err(anyhow!(
                "Insufficient storage space (additional {required} bytes required)"
            ));
        }

        Ok(())
    }
}

#[async_trait]
impl ParallelUploader for SHDWMethod {
    fn upload_asset(&self, asset_info: AssetInfo) -> JoinHandle<Result<(String, String)>> {
        let config = self.0.clone();
        tokio::spawn(async move { config.send(asset_info).await })
    }
}

impl Config {
    async fn send(&self, asset_info: AssetInfo) -> Result<(String, String)> {
        let data = match asset_info.data_type {
            DataType::Image => fs::read(&asset_info.content)?,
            DataType::Metadata => asset_info.content.into_bytes(),
            DataType::Animation => fs::read(&asset_info.content)?,
        };

        let mut context = Context::new(&SHA256);
        context.update(asset_info.name.as_bytes());
        let hash = HEXLOWER.encode(context.finish().as_ref());

        let message = format!(
            "Shadow Drive Signed Message:\n\
            Storage Account: {}\n\
            Upload files with hash: {hash}",
            self.storage_account
        );

        let signature = self.keypair.sign_message(message.as_bytes()).to_string();

        let mut form = Form::new();
        let file = Part::bytes(data)
            .file_name(asset_info.name.clone())
            .mime_str(asset_info.content_type.as_str())?;
        form = form
            .part("file", file)
            .text("message", signature)
            .text("overwrite", "true")
            .text("signer", self.keypair.pubkey().to_string())
            .text("storage_account", self.storage_account.to_string())
            .text("fileNames", asset_info.name.to_string());

        let http_client = reqwest::Client::new();
        let response = http_client
            .post(format!("{}/upload", self.endpoint))
            .multipart(form)
            .send()
            .await?;
        let status = response.status();

        if status.is_success() {
            Ok((
                asset_info.asset_id,
                format!(
                    "{SHDW_DRIVE_LOCATION}/{}/{}",
                    self.storage_account, asset_info.name
                ),
            ))
        } else {
            Err(anyhow!(UploadError::SendDataFailed(format!(
                "Error uploading file ({}): {}",
                status,
                response.text().await?,
            ))))
        }
    }
}
