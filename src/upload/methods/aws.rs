use std::{fs, sync::Arc};

use async_trait::async_trait;
use aws_sdk_s3::{types::ByteStream, Client};
use bs58;
use tokio::task::JoinHandle;

use crate::{
    common::*,
    config::*,
    upload::{
        assets::{AssetPair, DataType},
        uploader::{AssetInfo, ParallelUploader, Prepare},
    },
};

pub struct AWSMethod {
    pub aws_client: Arc<Client>,
    pub bucket: String,
}

impl AWSMethod {
    pub async fn new(config_data: &ConfigData) -> Result<Self> {
        let shared_config = aws_config::load_from_env().await;
        let client = Client::new(&shared_config);

        if let Some(aws_s3_bucket) = &config_data.aws_s3_bucket {
            Ok(Self {
                aws_client: Arc::new(client),
                bucket: aws_s3_bucket.to_string(),
            })
        } else {
            Err(anyhow!("Missing 'awsS3Bucket' value in config file."))
        }
    }

    async fn send(
        client: Arc<Client>,
        bucket: String,
        asset_info: AssetInfo,
    ) -> Result<(String, String)> {
        let data = match asset_info.data_type {
            DataType::Image => fs::read(&asset_info.content)?,
            DataType::Metadata => asset_info.content.into_bytes(),
            DataType::Animation => fs::read(&asset_info.content)?,
        };

        let key = bs58::encode(&asset_info.name).into_string();

        client
            .put_object()
            .bucket(&bucket)
            .key(&key)
            .body(ByteStream::from(data))
            .content_type(asset_info.content_type)
            .send()
            .await?;

        let link = format!("https://{}.s3.amazonaws.com/{}", bucket, key);

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
        let client = self.aws_client.clone();
        let bucket = self.bucket.clone();
        tokio::spawn(async move { AWSMethod::send(client, bucket, asset_info).await })
    }
}
