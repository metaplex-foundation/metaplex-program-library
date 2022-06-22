use async_trait::async_trait;
use console::style;
use futures::future::select_all;
use reqwest::{header, Client, StatusCode};
use std::{
    cmp,
    collections::HashSet,
    ffi::OsStr,
    fs,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::time::{sleep, Duration};

use crate::{common::*, config::*, upload::*, utils::*};

const NFT_STORAGE_API_URL: &str = "https://api.nft.storage";
const NFT_STORAGE_GATEWAY_URL: &str = "https://nftstorage.link/ipfs";
// Request time window (ms) to avoid the rate limit.
const REQUEST_WAIT: u64 = 1000;
// Number of concurrent requests.
const LIMIT: usize = 1;
// Response timeout (seconds).
const TIMEOUT: u64 = 20;

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

struct UploadInfo {
    asset_id: String,
    file_path: String,
    image_link: String,
    data_type: DataType,
    animation_link: Option<String>,
}

pub struct NftStorageHandler {
    client: Arc<Client>,
}

impl NftStorageHandler {
    /// Initialize a new NftStorageHandler.
    pub async fn initialize(config_data: &ConfigData) -> Result<NftStorageHandler> {
        if let Some(auth_token) = &config_data.nft_storage_auth_token {
            let client_builder = Client::builder();

            let mut headers = header::HeaderMap::new();
            let bearer_value = format!("Bearer {}", auth_token);
            let mut auth_value = header::HeaderValue::from_str(&bearer_value)?;
            auth_value.set_sensitive(true);
            headers.insert(header::AUTHORIZATION, auth_value);

            let client = client_builder
                .default_headers(headers)
                .timeout(Duration::from_secs(TIMEOUT))
                .build()?;

            let url = format!("{}/", NFT_STORAGE_API_URL);
            let response = client.get(url).send().await?;

            match response.status() {
                StatusCode::OK => Ok(NftStorageHandler {
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

    /// Send an file to Nft Storage and wait for a response.
    async fn send_to_nft_storage(
        client: Arc<Client>,
        info: UploadInfo,
    ) -> Result<(String, String)> {
        let data = match info.data_type {
            DataType::Image => fs::read(&info.file_path)?,
            DataType::Metadata => {
                // replaces the image link without modifying the original file to avoid
                // changing the hash of the metadata file
                get_updated_metadata(&info.file_path, &info.image_link, info.animation_link)?
                    .into_bytes()
            }
            DataType::Animation => fs::read(&info.file_path)?,
        };

        let url = format!("{}/upload", NFT_STORAGE_API_URL);
        let response = client.post(url).body(data).send().await?;
        let status = response.status().is_success();
        let body = response.json::<Value>().await?;

        match status {
            true => {
                let StoreNftResponse {
                    value: NftValue { cid },
                    ..
                }: StoreNftResponse = serde_json::from_value(body)?;

                Ok((info.asset_id, cid))
            }
            false => Err(anyhow!(
                "File upload to NFT Storage Failed: {}",
                info.asset_id
            )),
        }
    }
}

#[async_trait]
impl UploadHandler for NftStorageHandler {
    /// Nothing to do, Nft Storage ready for the upload.
    async fn prepare(
        &self,
        _sugar_config: &SugarConfig,
        _assets: &HashMap<usize, AssetPair>,
        _image_indices: &[usize],
        _metadata_indices: &[usize],
        _animation_indices: &[usize],
    ) -> Result<()> {
        Ok(())
    }

    /// Upload the data to Nft Storage
    async fn upload_data(
        &self,
        _sugar_config: &SugarConfig,
        assets: &HashMap<usize, AssetPair>,
        cache: &mut Cache,
        indices: &[usize],
        data_type: DataType,
        interrupted: Arc<AtomicBool>,
    ) -> Result<Vec<UploadError>> {
        let mut extension = HashSet::with_capacity(1);
        let mut paths = Vec::new();

        for index in indices {
            let item = match assets.get(index) {
                Some(asset_index) => asset_index,
                None => return Err(anyhow::anyhow!("Failed to get asset at index {}", index)),
            };
            // chooses the file path based on the data type
            let file_path = match data_type {
                DataType::Image => item.image.clone(),
                DataType::Metadata => item.metadata.clone(),
                DataType::Animation => item.animation.clone().unwrap(),
            };

            let path = Path::new(&file_path);
            let ext = path
                .extension()
                .and_then(OsStr::to_str)
                .expect("Failed to convert path extension to valid unicode.");
            extension.insert(String::from(ext));

            paths.push(file_path);
        }

        println!("\nSending data: (Ctrl+C to abort)");

        let pb = progress_bar_with_style(paths.len() as u64);
        let mut objects = Vec::new();

        for file_path in paths {
            // path to the image/metadata file
            let path = Path::new(&file_path);
            // id of the asset (to be used to update the cache link)
            let asset_id = String::from(
                path.file_stem()
                    .and_then(OsStr::to_str)
                    .expect("Failed to get convert path file ext to valid unicode."),
            );

            let cache_item = match cache.items.0.get(&asset_id) {
                Some(item) => item,
                None => {
                    return Err(anyhow::anyhow!(
                        "Failed to get config item at index: {}",
                        asset_id
                    ))
                }
            };

            objects.push(UploadInfo {
                asset_id: asset_id.to_string(),
                file_path: String::from(
                    path.to_str().expect("Failed to convert path from unicode."),
                ),
                image_link: cache_item.image_link.clone(),
                data_type: data_type.clone(),
                animation_link: cache_item.animation_link.clone(),
            });
        }

        let mut handles = Vec::new();

        for object in objects.drain(0..cmp::min(objects.len(), LIMIT)) {
            let client = self.client.clone();
            handles.push(tokio::spawn(async move {
                NftStorageHandler::send_to_nft_storage(client, object).await
            }));
        }

        let mut errors = Vec::new();

        while !interrupted.load(Ordering::SeqCst) && !handles.is_empty() {
            match select_all(handles).await {
                (Ok(res), _index, remaining) => {
                    // independently if the upload was successful or not
                    // we continue to try the remaining ones
                    handles = remaining;

                    if res.is_ok() {
                        let val = res?;
                        let link = format!("{}/{}", NFT_STORAGE_GATEWAY_URL, val.1);
                        // cache item to update
                        let item = cache.items.0.get_mut(&val.0).unwrap();

                        match data_type {
                            DataType::Image => item.image_link = link,
                            DataType::Metadata => item.metadata_link = link,
                            DataType::Animation => item.animation_link = Some(link),
                        }
                        // updates the progress bar
                        pb.inc(1);
                    } else {
                        // user will need to retry the upload
                        errors.push(UploadError::SendDataFailed(format!(
                            "Nft Storage upload error: {:?}",
                            res.err().unwrap()
                        )));
                    }
                }
                (Err(err), _index, remaining) => {
                    errors.push(UploadError::SendDataFailed(format!(
                        "Nft Storage upload error: {:?}",
                        err
                    )));
                    // ignoring all errors
                    handles = remaining;
                }
            }

            if !objects.is_empty() {
                // if we are done, let spawn more transactions
                if handles.is_empty() {
                    // syncs cache (checkpoint)
                    cache.sync_file()?;
                    // minimum gap between request
                    sleep(Duration::from_millis(REQUEST_WAIT)).await;

                    for object in objects.drain(0..cmp::min(objects.len(), LIMIT)) {
                        let client = self.client.clone();
                        handles.push(tokio::spawn(async move {
                            NftStorageHandler::send_to_nft_storage(client, object).await
                        }));
                    }
                }
            }
        }

        if !errors.is_empty() {
            pb.abandon_with_message(format!("{}", style("Upload failed ").red().bold()));
        } else if !objects.is_empty() {
            pb.abandon_with_message(format!("{}", style("Upload aborted ").red().bold()));
            return Err(
                UploadError::SendDataFailed("Not all files were uploaded.".to_string()).into(),
            );
        } else {
            pb.finish_with_message(format!("{}", style("Upload successful ").green().bold()));
        }

        // makes sure the cache file is updated
        cache.sync_file()?;

        Ok(errors)
    }
}
