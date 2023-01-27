use std::{fs, ops::Deref, path::Path, sync::Arc};

use async_trait::async_trait;
use reqwest::{
    header,
    multipart::{Form, Part},
    Client, StatusCode,
};
use tokio::task::JoinHandle;

use crate::{common::*, config::*, upload::*};

// API end point.
const UPLOAD_ENDPOINT: &str = "/pinning/pinFileToIPFS";
// Storage end point.
const AUTH_TEST_URL: &str = "https://api.pinata.cloud/data/testAuthentication";
// File size limit (10mb).
const FILE_SIZE_LIMIT: u64 = 10 * 1024 * 1024;

/// response after an nft was stored
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct PinataResponse {
    /// Hash of the upload
    pub ipfs_hash: String,
}

pub struct Config {
    client: Client,
    endpoint: String,
    content_gateway: String,
    parallel_limit: u16,
}

pub struct PinataMethod(Arc<Config>);

impl Deref for PinataMethod {
    type Target = Arc<Config>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PinataMethod {
    /// Initialize a new PinataMethod.
    pub async fn new(config_data: &ConfigData) -> Result<Self> {
        if let Some(pinata_config) = &config_data.pinata_config {
            let client_builder = Client::builder();

            let mut headers = header::HeaderMap::new();
            let bearer_value = format!("Bearer {}", &pinata_config.jwt);
            let mut auth_value = header::HeaderValue::from_str(&bearer_value)?;
            auth_value.set_sensitive(true);
            headers.insert(header::AUTHORIZATION, auth_value);

            let client = client_builder.default_headers(headers).build()?;
            // always use the public gateway for the authentication
            let response = client.get(AUTH_TEST_URL).send().await?;

            match response.status() {
                StatusCode::OK => {
                    // upload endpoint
                    let endpoint_url =
                        url::Url::parse(&pinata_config.api_gateway)?.join(UPLOAD_ENDPOINT)?;

                    // maximum number of concurrent uploads
                    let parallel_limit = if let Some(parallel_limit) = pinata_config.parallel_limit
                    {
                        parallel_limit
                    } else {
                        PARALLEL_LIMIT as u16
                    };

                    Ok(Self(Arc::new(Config {
                        client,
                        endpoint: endpoint_url.to_string(),
                        content_gateway: pinata_config.content_gateway.clone(),
                        parallel_limit,
                    })))
                }
                StatusCode::UNAUTHORIZED => Err(anyhow!("Invalid pinata JWT token.")),
                code => Err(anyhow!("Could not initialize pinata client: {code}")),
            }
        } else {
            Err(anyhow!("Missing 'pinataConfig' in config file."))
        }
    }
}

#[async_trait]
impl Prepare for PinataMethod {
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
                        "File '{}' exceeds the current 10MB file size limit",
                        item.name,
                    ));
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl ParallelUploader for PinataMethod {
    /// Returns the number of files that to be send in parallel.
    fn parallel_limit(&self) -> usize {
        self.parallel_limit as usize
    }

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

        let mut form = Form::new();

        let file = Part::bytes(data)
            .file_name(asset_info.name.clone())
            .mime_str(asset_info.content_type.as_str())?;
        form = form
            .part("file", file)
            .text("pinataOptions", "{\"wrapWithDirectory\": true}");

        let response = self
            .client
            .post(&self.endpoint)
            .multipart(form)
            .send()
            .await?;
        let status = response.status();

        if status.is_success() {
            let body = response.json::<Value>().await?;
            let PinataResponse { ipfs_hash } = serde_json::from_value(body)?;

            let uri = url::Url::parse(&self.content_gateway)?
                .join(&format!("/ipfs/{}/{}", ipfs_hash, asset_info.name))?;

            Ok((asset_info.asset_id, uri.to_string()))
        } else {
            let body = response.json::<Value>().await?;
            let details = if let Some(details) = &body["error"]["details"].as_str() {
                details.to_string()
            } else {
                body.to_string()
            };
            Err(anyhow!(UploadError::SendDataFailed(format!(
                "Error uploading batch ({status}): {details}",
            ))))
        }
    }
}
