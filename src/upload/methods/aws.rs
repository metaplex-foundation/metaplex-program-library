use std::{fs, sync::Arc};

use async_trait::async_trait;
use bs58;
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
}

impl AWSMethod {
    pub async fn new(config_data: &ConfigData) -> Result<Self> {
        let credentials = Credentials::default()?;
        let region = AWSMethod::load_region()?;

        if let Some(aws_s3_bucket) = &config_data.aws_s3_bucket {
            Ok(Self {
                bucket: Arc::new(Bucket::new(aws_s3_bucket, region, credentials)?),
            })
        } else {
            Err(anyhow!("Missing 'awsS3Bucket' value in config file."))
        }
    }

    fn load_region() -> Result<Region> {
        let home_dir = dirs::home_dir().expect("Couldn't find home dir.");
        let credentials = home_dir.join(Path::new(".aws/credentials"));
        let configuration = ini!(credentials
            .to_str()
            .ok_or_else(|| anyhow!("Failed to load AWS credentials"))?);

        if let Some(region) = &configuration["default"]["region"] {
            Ok(region.parse()?)
        } else {
            Err(anyhow!("Region configuration not found"))
        }
    }

    async fn send(bucket: Arc<Bucket>, asset_info: AssetInfo) -> Result<(String, String)> {
        let data = match asset_info.data_type {
            DataType::Image => fs::read(&asset_info.content)?,
            DataType::Metadata => asset_info.content.into_bytes(),
            DataType::Animation => fs::read(&asset_info.content)?,
        };

        let key = bs58::encode(&asset_info.name).into_string();
        let mut retry = MAX_RETRY;
        // send data to AWS S3 with a simple retry logic (mitigates dns lookup errors)
        loop {
            match bucket
                .put_object_with_content_type(&key, &data, &asset_info.content_type)
                .await
            {
                Ok(_) => break,
                Err(error) => {
                    if retry == 0 {
                        return Err(error.into());
                    }
                    // we try one more time before reporting the error
                    retry -= 1;
                }
            }
        }

        let link = format!("https://{}.s3.amazonaws.com/{}", bucket.name(), key);

        Ok((asset_info.asset_id, link))
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
        tokio::spawn(async move { AWSMethod::send(bucket, asset_info).await })
    }
}
