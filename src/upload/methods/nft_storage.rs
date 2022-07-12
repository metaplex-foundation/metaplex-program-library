use std::{
    fs,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use async_trait::async_trait;
use reqwest::{
    header,
    multipart::{Form, Part},
    Client, StatusCode,
};
use tokio::time::{sleep, Duration};

use crate::{common::*, config::*, upload::*};

// API end point.
const NFT_STORAGE_API_URL: &str = "https://api.nft.storage";
// Storage end point.
const NFT_STORAGE_GATEWAY_URL: &str = "https://nftstorage.link/ipfs";
// Request time window (ms) to avoid the rate limit.
const REQUEST_WAIT: u64 = 10000;
// File size limit (100mb).
const FILE_SIZE_LIMIT: u64 = 100 * 1024 * 1024;
// Number of files per request limit.
const FILE_COUNT_LIMIT: u64 = 100;

pub enum NftStorageError {
    ApiError(Value),
}

/// response after an nft was stored
#[derive(Debug, Deserialize, Default)]
pub struct StoreNftResponse {
    /// status of the request
    pub ok: bool,
    /// stored nft data
    pub value: NftValue,
}

/// main obj that hold all the response data
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct NftValue {
    /// ipfs cid (file hash)
    pub cid: String,
}

/// response after an error
#[derive(Debug, Deserialize, Default)]
pub struct StoreNftError {
    /// status of the request
    pub ok: bool,
    /// stored nft error
    pub error: NftError,
}

/// hold the error message
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct NftError {
    /// error message
    pub message: String,
}

pub struct NftStorageMethod {
    client: Arc<Client>,
}

impl NftStorageMethod {
    /// Initialize a new NftStorageHandler.
    pub async fn new(config_data: &ConfigData) -> Result<Self> {
        if let Some(auth_token) = &config_data.nft_storage_auth_token {
            let client_builder = Client::builder();

            let mut headers = header::HeaderMap::new();
            let bearer_value = format!("Bearer {}", auth_token);
            let mut auth_value = header::HeaderValue::from_str(&bearer_value)?;
            auth_value.set_sensitive(true);
            headers.insert(header::AUTHORIZATION, auth_value);

            let client = client_builder.default_headers(headers).build()?;

            let url = format!("{}/", NFT_STORAGE_API_URL);
            let response = client.get(url).send().await?;

            match response.status() {
                StatusCode::OK => Ok(Self {
                    client: Arc::new(client),
                }),
                StatusCode::UNAUTHORIZED => {
                    Err(anyhow!("Invalid nft.storage authentication token."))
                }
                code => Err(anyhow!("Could not initialize nft.storage client: {code}")),
            }
        } else {
            Err(anyhow!(
                "Missing 'nftStorageAuthToken' value in config file."
            ))
        }
    }
}

#[async_trait]
impl Prepare for NftStorageMethod {
    /// Verifies that no file is larger than 100MB (upload of files larger than 100MB are
    /// not currently supported).
    async fn prepare(
        &self,
        _sugar_config: &SugarConfig,
        asset_pairs: &HashMap<isize, AssetPair>,
        asset_indices: Vec<(DataType, &[isize])>,
    ) -> Result<()> {
        for (data_type, indices) in asset_indices {
            for index in indices {
                let item = asset_pairs.get(index).unwrap();
                let size = match data_type {
                    DataType::Image => {
                        let path = Path::new(&item.image);
                        fs::metadata(path)?.len()
                    }
                    DataType::Animation => {
                        if let Some(animation) = &item.animation {
                            let path = Path::new(animation);
                            fs::metadata(path)?.len()
                        } else {
                            0
                        }
                    }
                    DataType::Metadata => {
                        let mock_uri = "x".repeat(MOCK_URI_SIZE);
                        let animation = if item.animation.is_some() {
                            Some(mock_uri.clone())
                        } else {
                            None
                        };

                        get_updated_metadata(&item.metadata, &mock_uri.clone(), &animation)?
                            .into_bytes()
                            .len() as u64
                    }
                };

                if size > FILE_SIZE_LIMIT {
                    return Err(anyhow!(
                        "File '{}' exceeds the current 100MB file size limit",
                        item.name,
                    ));
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Uploader for NftStorageMethod {
    /// Upload the data to Nft Storage
    async fn upload(
        &self,
        _sugar_config: &SugarConfig,
        cache: &mut Cache,
        data_type: DataType,
        assets: &mut Vec<AssetInfo>,
        progress: &ProgressBar,
        interrupted: Arc<AtomicBool>,
    ) -> Result<Vec<UploadError>> {
        let mut batches: Vec<Vec<&AssetInfo>> = Vec::new();
        let mut current: Vec<&AssetInfo> = Vec::new();
        let mut upload_size = 0;
        let mut upload_count = 0;

        for asset_info in assets {
            let size = match data_type {
                DataType::Image | DataType::Animation => {
                    let path = Path::new(&asset_info.content);
                    fs::metadata(path)?.len()
                }
                DataType::Metadata => {
                    let content = String::from(&asset_info.content);
                    content.into_bytes().len() as u64
                }
            };

            if (upload_size + size) > FILE_SIZE_LIMIT || (upload_count + 1) > FILE_COUNT_LIMIT {
                batches.push(current);
                current = Vec::new();
                upload_size = 0;
                upload_count = 0;
            }

            upload_size += size;
            upload_count += 1;
            current.push(asset_info);
        }
        // adds the last chunk (if there is one)
        if !current.is_empty() {
            batches.push(current);
        }

        let mut errors = Vec::new();
        // sets the length of the progress bar as the number of batches
        progress.set_length(batches.len() as u64);

        while !interrupted.load(Ordering::SeqCst) && !batches.is_empty() {
            let batch = batches.remove(0);
            let mut form = Form::new();

            for asset_info in &batch {
                let data = match asset_info.data_type {
                    DataType::Image | DataType::Animation => fs::read(&asset_info.content)?,
                    DataType::Metadata => {
                        let content = String::from(&asset_info.content);
                        content.into_bytes()
                    }
                };

                let file = Part::bytes(data)
                    .file_name(asset_info.name.clone())
                    .mime_str(asset_info.content_type.as_str())?;
                form = form.part("file", file);
            }

            let response = self
                .client
                .post(format!("{NFT_STORAGE_API_URL}/upload"))
                .multipart(form)
                .send()
                .await?;
            let status = response.status();

            if status.is_success() {
                let body = response.json::<Value>().await?;
                let StoreNftResponse {
                    value: NftValue { cid },
                    ..
                }: StoreNftResponse = serde_json::from_value(body)?;

                // updates the cache content

                for asset_info in batch {
                    let id = asset_info.asset_id.clone();
                    let uri = format!("{NFT_STORAGE_GATEWAY_URL}/{cid}/{}", asset_info.name);
                    // cache item to update
                    let item = cache.items.get_mut(&id).unwrap();

                    match data_type {
                        DataType::Image => item.image_link = uri,
                        DataType::Metadata => item.metadata_link = uri,
                        DataType::Animation => item.animation_link = Some(uri),
                    }
                }
                // syncs cache (checkpoint)
                cache.sync_file()?;
                // updates the progress bar
                progress.inc(1);
            } else {
                let body = response.json::<Value>().await?;
                let StoreNftError {
                    error: NftError { message },
                    ..
                }: StoreNftError = serde_json::from_value(body)?;

                errors.push(UploadError::SendDataFailed(format!(
                    "Error uploading batch ({}): {}",
                    status, message
                )));
            }
            if !batches.is_empty() {
                // wait to minimize the chance of getting caught by the rate limit
                sleep(Duration::from_millis(REQUEST_WAIT)).await;
            }
        }

        Ok(errors)
    }
}
