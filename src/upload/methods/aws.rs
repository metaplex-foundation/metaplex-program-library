use std::{fs, sync::Arc};

use async_trait::async_trait;
use ini::ini;
use s3::{bucket::Bucket, creds::Credentials, region::Region};
use tokio::task::JoinHandle;

use crate::{
    common::*,
    config::*,
    upload::{
        assets::{AssetPair, DataType},
        uploader::{AssetInfo, ParallelUploader, Prepare},
    },
};

// Maximum number of times to retry each individual upload.
const MAX_RETRY: u8 = 3;

pub struct AWSMethod {
    pub bucket: Arc<Bucket>,
    pub directory: String,
    pub domain: String,
}

impl AWSMethod {
    pub async fn new(config_data: &ConfigData) -> Result<Self> {
        let profile = &config_data
            .aws_config
            .as_ref()
            .ok_or_else(|| anyhow!("AWS values not specified in config file!"))?
            .profile;

        let credentials = Credentials::from_profile(Some(profile))?;
        let region = AWSMethod::load_region(config_data)?;

        if let Some(config) = &config_data.aws_config {
            let domain = if let Some(domain) = &config.domain {
                match url::Url::parse(domain) {
                    Ok(url) => url.to_string(),
                    Err(error) => {
                        return Err(anyhow!("Malformed domain URL ({})", error.to_string()))
                    }
                }
            } else {
                format!("https://{}.s3.amazonaws.com", &config.bucket)
            };

            Ok(Self {
                bucket: Arc::new(Bucket::new(&config.bucket, region, credentials)?),
                directory: config.directory.clone(),
                domain,
            })
        } else {
            Err(anyhow!("Missing AwsConfig 'bucket' value in config file."))
        }
    }

    fn load_region(config_data: &ConfigData) -> Result<Region> {
        let home_dir = dirs::home_dir().expect("Couldn't find home dir.");
        let credentials = home_dir.join(Path::new(".aws/credentials"));
        let configuration = ini!(credentials
            .to_str()
            .ok_or_else(|| anyhow!("Failed to load AWS credentials"))?);

        let profile = &config_data
            .aws_config
            .as_ref()
            .ok_or_else(|| anyhow!("AWS values not specified in config file!"))?
            .profile;

        let region = &configuration
            .get(profile)
            .ok_or_else(|| anyhow!("Profile not found in AWS credentials file!"))?
            .get("region")
            .ok_or_else(|| anyhow!("Region not found in AWS credentials file!"))?
            .as_ref()
            .ok_or_else(|| anyhow!("Region not found in AWS credentials file!"))?
            .to_string();

        Ok(region.parse()?)
    }

    async fn send(
        bucket: Arc<Bucket>,
        directory: String,
        domain: String,
        asset_info: AssetInfo,
    ) -> Result<(String, String)> {
        let data = match asset_info.data_type {
            DataType::Image => fs::read(&asset_info.content)?,
            DataType::Metadata => asset_info.content.into_bytes(),
            DataType::Animation => fs::read(&asset_info.content)?,
        };

        // Take care of any spaces in the directory path.
        let directory = directory.replace(' ', "_");

        let path = Path::new(&directory).join(&asset_info.name);
        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow!("Failed to convert S3 bucket directory path to string."))?;

        let mut retry = MAX_RETRY;
        // send data to AWS S3 with a simple retry logic (mitigates dns lookup errors)
        loop {
            match bucket
                .put_object_with_content_type(path_str, &data, &asset_info.content_type)
                .await
            {
                Ok((_, code)) => match code {
                    200 => {
                        break;
                    }
                    _ => {
                        return Err(anyhow!(
                            "Failed to upload {} to S3 with Http Code: {code}",
                            asset_info.name
                        ));
                    }
                },
                Err(error) => {
                    if retry == 0 {
                        return Err(error.into());
                    }
                    // we try one more time before reporting the error
                    retry -= 1;
                }
            }
        }

        let link = url::Url::parse(&domain)?.join(path_str)?;

        Ok((asset_info.asset_id, link.to_string()))
    }
}

#[async_trait]
impl Prepare for AWSMethod {
    async fn prepare(
        &self,
        _sugar_config: &SugarConfig,
        _asset_pairs: &HashMap<isize, AssetPair>,
        _asset_indices: Vec<(DataType, &[isize])>,
    ) -> Result<()> {
        // nothing to do here
        Ok(())
    }
}

#[async_trait]
impl ParallelUploader for AWSMethod {
    fn upload_asset(&self, asset_info: AssetInfo) -> JoinHandle<Result<(String, String)>> {
        let bucket = self.bucket.clone();
        let directory = self.directory.clone();
        let domain = self.domain.clone();

        tokio::spawn(async move { AWSMethod::send(bucket, directory, domain, asset_info).await })
    }
}
