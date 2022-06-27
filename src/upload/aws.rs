use async_trait::async_trait;
use aws_sdk_s3::{types::ByteStream, Client};
use bs58;
use console::style;
use futures::future::select_all;
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

use crate::{common::*, config::*, constants::PARALLEL_LIMIT, upload::*, utils::*};

struct ObjectInfo {
    asset_id: String,
    file_path: String,
    image_link: String,
    data_type: DataType,
    content_type: String,
    bucket: String,
    animation_link: Option<String>,
}

pub struct AWSHandler {
    client: Arc<Client>,
    bucket: String,
}

impl AWSHandler {
    /// Initialize a new AWSHandler.
    pub async fn initialize(config_data: &ConfigData) -> Result<AWSHandler> {
        let shared_config = aws_config::load_from_env().await;
        let client = Client::new(&shared_config);

        if let Some(aws_s3_bucket) = &config_data.aws_s3_bucket {
            Ok(AWSHandler {
                client: Arc::new(client),
                bucket: aws_s3_bucket.to_string(),
            })
        } else {
            Err(anyhow!("Missing 'awsS3Bucket' value in config file."))
        }
    }

    /// Send an object to AWS and wait for a response.
    async fn send_to_aws(aws_client: Arc<Client>, info: ObjectInfo) -> Result<(String, String)> {
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

        let key = bs58::encode(&info.file_path).into_string();

        aws_client
            .put_object()
            .bucket(info.bucket)
            .key(&key)
            .body(ByteStream::from(data))
            .content_type(info.content_type)
            .send()
            .await?;

        Ok((info.asset_id, key))
    }
}

#[async_trait]
impl UploadHandler for AWSHandler {
    /// Nothing to do, AWS client ready for the upload.
    async fn prepare(
        &self,
        _sugar_config: &SugarConfig,
        _assets: &HashMap<isize, AssetPair>,
        _image_indices: &[isize],
        _metadata_indices: &[isize],
        _animation_indices: &[isize],
    ) -> Result<()> {
        Ok(())
    }

    /// Upload the data to AWS S3.
    async fn upload_data(
        &self,
        _sugar_config: &SugarConfig,
        assets: &HashMap<isize, AssetPair>,
        cache: &mut Cache,
        indices: &[isize],
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

        // validates that all files have the same extension
        let extension = if extension.len() == 1 {
            extension.iter().next().unwrap()
        } else {
            return Err(anyhow!("Invalid file extension: {:?}", extension));
        };

        let content_type = match data_type {
            DataType::Image => format!("image/{}", extension),
            DataType::Metadata => "application/json".to_string(),
            DataType::Animation => format!("video/{}", extension),
        };

        println!("\nSending data: (Ctrl+C to abort)");

        let pb = progress_bar_with_style(paths.len() as u64);
        let mut objects = Vec::new();

        for file_path in paths {
            // path to the image/metadata file
            let path = Path::new(&file_path);

            let (asset_id, cache_item) = get_cache_item(path, cache)?;

            objects.push(ObjectInfo {
                asset_id: asset_id.to_string(),
                file_path: String::from(
                    path.to_str().expect("Failed to convert path from unicode."),
                ),
                image_link: cache_item.image_link.clone(),
                data_type: data_type.clone(),
                content_type: content_type.clone(),
                bucket: self.bucket.clone(),
                animation_link: cache_item.animation_link.clone(),
            });
        }

        let mut handles = Vec::new();

        for object in objects.drain(0..cmp::min(objects.len(), PARALLEL_LIMIT)) {
            let aws_client = self.client.clone();
            handles.push(tokio::spawn(async move {
                AWSHandler::send_to_aws(aws_client, object).await
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
                        let link = format!("https://{}.s3.amazonaws.com/{}", self.bucket, val.1);
                        // cache item to update
                        let item = cache.items.get_mut(&val.0).unwrap();

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
                            "AWS upload error: {:?}",
                            res.err().unwrap()
                        )));
                    }
                }
                (Err(err), _index, remaining) => {
                    errors.push(UploadError::SendDataFailed(format!(
                        "AWS upload error: {:?}",
                        err
                    )));
                    // ignoring all errors
                    handles = remaining;
                }
            }

            if !objects.is_empty() {
                // if we are half way through, let spawn more transactions
                if (PARALLEL_LIMIT - handles.len()) > (PARALLEL_LIMIT / 2) {
                    // syncs cache (checkpoint)
                    cache.sync_file()?;

                    for object in objects.drain(0..cmp::min(objects.len(), PARALLEL_LIMIT / 2)) {
                        let aws_client = self.client.clone();
                        handles.push(tokio::spawn(async move {
                            AWSHandler::send_to_aws(aws_client, object).await
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
