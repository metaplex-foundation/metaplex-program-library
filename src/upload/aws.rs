use async_trait::async_trait;
use aws_sdk_s3::{types::ByteStream, Client};
use bs58;
use console::style;
use futures::future::select_all;
use std::{cmp, collections::HashSet, ffi::OsStr, fs, path::Path, sync::Arc};

use crate::{common::*, config::*, constants::PARALLEL_LIMIT, upload::*, utils::*};

struct ObjectInfo {
    asset_id: String,
    file_path: String,
    media_link: String,
    data_type: DataType,
    content_type: String,
    aws_client: Arc<Client>,
    bucket: String,
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
    async fn send_to_aws(info: ObjectInfo) -> Result<(String, String)> {
        let data = match info.data_type {
            DataType::Media => fs::read(&info.file_path)?,
            DataType::Metadata => {
                // replaces the media link without modifying the original file to avoid
                // changing the hash of the metadata file
                get_updated_metadata(&info.file_path, &info.media_link)
                    .unwrap()
                    .into_bytes()
            }
        };

        let key = bs58::encode(&info.file_path).into_string();

        info.aws_client
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
    /// Upload the data to AWS S3.
    async fn upload_data(
        &self,
        _sugar_config: &SugarConfig,
        assets: &HashMap<usize, AssetPair>,
        cache: &mut Cache,
        indices: &[usize],
        data_type: DataType,
    ) -> Result<Vec<UploadError>> {
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

        let content_type = match data_type {
            DataType::Media => format!("image/{}", extension),
            DataType::Metadata => "application/json".to_string(),
        };

        println!("\nSending data: (Ctrl+C to abort)");

        let pb = progress_bar_with_style(paths.len() as u64);
        let mut objects = Vec::new();

        for file_path in paths {
            // path to the media/metadata file
            let path = Path::new(&file_path);
            // id of the asset (to be used to update the cache link)
            let asset_id = String::from(path.file_stem().and_then(OsStr::to_str).unwrap());
            let cache_item = cache.items.0.get(&asset_id).unwrap();

            objects.push(ObjectInfo {
                asset_id: asset_id.to_string(),
                file_path: String::from(path.to_str().unwrap()),
                media_link: cache_item.media_link.clone(),
                data_type: data_type.clone(),
                content_type: content_type.clone(),
                aws_client: self.client.clone(),
                bucket: self.bucket.clone(),
            });
        }

        let mut handles = Vec::new();

        for object in objects.drain(0..cmp::min(objects.len(), PARALLEL_LIMIT)) {
            handles.push(tokio::spawn(async move {
                AWSHandler::send_to_aws(object).await
            }));
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
                        let link = format!(
                            "https://{}.s3.amazonaws.com/{}",
                            self.bucket,
                            val.1
                        );
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
                    for object in objects.drain(0..cmp::min(objects.len(), PARALLEL_LIMIT / 2)) {
                        handles.push(tokio::spawn(async move {
                            AWSHandler::send_to_aws(object).await
                        }));
                    }
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
