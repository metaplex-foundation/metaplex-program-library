use std::{
    cmp,
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::Result;
use async_trait::async_trait;
use console::style;
use futures::future::select_all;
pub use indicatif::ProgressBar;
use tokio::task::JoinHandle;

use crate::{
    cache::Cache,
    config::{ConfigData, SugarConfig, UploadMethod},
    constants::PARALLEL_LIMIT,
    upload::{
        assets::{AssetPair, DataType},
        methods::*,
        UploadError,
    },
};

// Size of the mock media URI for cost calculations.
pub const MOCK_URI_SIZE: usize = 100;

/// Struct representing an asset ready for upload. An `AssetInfo` can represent
/// a physical file, in which case the `content` will correspond to the name
/// of the file; or an in-memory asset, in which case the `content` will correspond
/// to the content of the asset.
///
/// For example, for image files, the `content` contains the path of the file on the
/// file system. In the case of json metadata files, the `content` contains the string
/// representation of the json metadata.
///
pub struct AssetInfo {
    /// Id of the asset in the cache.
    pub asset_id: String,
    /// Name (file name) of the asset.
    pub name: String,
    /// Content of the asset - either a file path of the string representation fo the content.
    pub content: String,
    /// Type of the asset.
    pub data_type: DataType,
    /// MIME content type.
    pub content_type: String,
}

/// Types that can be prepared to upload assets (files).
///
/// All implementation of [`Uploader`](Uploader) need to implement this trait.
#[async_trait]
pub trait Prepare {
    /// Prepare the upload of the specified media/metadata files, e.g.:
    /// - check if any file exceeds a size limit;
    /// - check if there is storage space for the upload;
    /// - check/add funds for the upload.
    ///
    /// The `prepare` receives the information of all files that will be upload.
    ///
    /// # Arguments
    ///
    /// * `sugar_config` - The current sugar configuration
    /// * `asset_pairs` - Mapping of `index` to an `AssetPair`
    /// * `asset_indices` - Vector with the information of which asset pair indices will be upload grouped by type.
    ///
    /// The `asset_pairs` contain the complete information of the assets, but only the assets specified in the
    /// `asset_indices` will be uploaded. E.g., if index `1` is only present in the `DataType::Image` indices' array,
    /// only the image of asset `1` will the uploaded.
    ///
    async fn prepare(
        &self,
        sugar_config: &SugarConfig,
        asset_pairs: &HashMap<isize, AssetPair>,
        asset_indices: Vec<(DataType, &[isize])>,
    ) -> Result<()>;
}

/// Types that can upload assets (files).
///
/// This trait should be implemented directly by upload methods that require full control on how the upload
/// is performed. For methods that support parallel uploads (threading), consider implementing
/// [`ParallelUploader`](ParallelUploader) instead.
///
#[async_trait]
pub trait Uploader: Prepare {
    /// Returns a vector [`UploadError`](super::errors::UploadError) with the errors (if any) after uploading all
    /// assets to the storage.
    ///
    /// This function will be called to upload each type of asset separately.
    ///
    /// # Arguments
    ///
    /// * `sugar_config` - The current sugar configuration
    /// * `cache` - Asset [`cache`](crate::cache::Cache) object (mutable)
    /// * `data_type` - Type of the asset being uploaded
    /// * `assets` - Vector of [`assets`](AssetInfo) to upload (mutable)
    /// * `progress` - Reference to the [`progress bar`](indicatif::ProgressBar) to provide feedback to
    ///                the console
    /// * `interrupted` - Reference to the shared interruption handler [`flag`](std::sync::atomic::AtomicBool)
    ///                   to receive notifications
    ///
    /// # Examples
    ///
    /// Implementations are expected to use the `interrupted` to control when the user aborts the upload process.
    /// In general, this would involve using it as a control of a loop:
    ///
    /// ```ignore
    /// while !interrupted.load(Ordering::SeqCst) {
    ///     // continue with the upload
    /// }
    /// ```
    ///
    /// After uploading an asset, its information need to be updated in the cache and the cache
    /// [`sync`](crate::cache::Cache#method.sync_file)ed to the file system. Syncing the cache to the file system
    /// might be slow for large collections, therefore it should be done as frequent as practical to avoid slowing
    /// down the upload process and, at the same time, minimizing the chances of information loss in case
    /// the user aborts the upload.
    ///
    /// ```ignore
    /// ...
    /// // once an asset has been upload
    ///
    /// let id = asset_info.asset_id.clone();
    /// let uri = "URI of the asset after upload";
    /// // cache item to update
    /// let item = cache.items.get_mut(&id).unwrap();
    ///
    /// match data_type {
    ///     DataType::Image => item.image_link = uri,
    ///     DataType::Metadata => item.metadata_link = uri,
    ///     DataType::Animation => item.animation_link = Some(uri),
    /// }
    /// // updates the progress bar
    /// progress.inc(1);
    ///
    /// ...
    ///
    /// // after several uploads
    /// cache.sync_file()?;
    /// ```
    ///
    async fn upload(
        &self,
        sugar_config: &SugarConfig,
        cache: &mut Cache,
        data_type: DataType,
        assets: &mut Vec<AssetInfo>,
        progress: &ProgressBar,
        interrupted: Arc<AtomicBool>,
    ) -> Result<Vec<UploadError>>;
}

/// Types that can upload assets in parallel.
///
/// This trait abstracts the threading logic and allows methods to focus on the logic of uploading a single
/// asset (file).
#[async_trait]
pub trait ParallelUploader: Uploader + Send + Sync {
    /// Returns a [`JoinHandle`](tokio::task::JoinHandle) to the task responsible to upload the specified asset.
    ///
    /// # Arguments
    ///
    /// * `asset` - The [`asset`](AssetInfo) to upload
    ///
    /// # Example
    ///
    /// In most cases, the function will return the value from [`tokio::spawn`](tokio::spawn):
    ///
    /// ```ignore
    /// tokio::spawn(async move {
    ///     // code responsible to upload a single asset
    /// });
    /// ```
    ///
    fn upload_asset(&self, asset: AssetInfo) -> JoinHandle<Result<(String, String)>>;
}

/// Default implementation of the trait ['Uploader'](Uploader) for all ['ParallelUploader'](ParallelUploader).
///
#[async_trait]
impl<T: ParallelUploader> Uploader for T {
    /// Uploads assets in parallel. It creates `PARALLEL_LIMIT`[PARALLEL_LIMIT] tasks at a time to avoid
    /// reaching the limit of concurrent files open and it syncs the cache file at every `PARALLEL_LIMIT / 2`
    /// step.
    ///
    async fn upload(
        &self,
        _sugar_config: &SugarConfig,
        cache: &mut Cache,
        data_type: DataType,
        assets: &mut Vec<AssetInfo>,
        progress: &ProgressBar,
        interrupted: Arc<AtomicBool>,
    ) -> Result<Vec<UploadError>> {
        let mut handles = Vec::new();

        for task in assets.drain(0..cmp::min(assets.len(), PARALLEL_LIMIT)) {
            handles.push(self.upload_asset(task));
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
                        let link = val.clone().1;
                        // cache item to update
                        let item = cache.items.0.get_mut(&val.0).unwrap();
                        match data_type {
                            DataType::Image => item.image_link = link,
                            DataType::Metadata => item.metadata_link = link,
                            DataType::Animation => item.animation_link = Some(link),
                        }
                        // updates the progress bar
                        progress.inc(1);
                    } else {
                        // user will need to retry the upload
                        errors.push(UploadError::SendDataFailed(format!(
                            "Upload error: {:?}",
                            res.err().unwrap()
                        )));
                    }
                }
                (Err(err), _index, remaining) => {
                    errors.push(UploadError::SendDataFailed(format!(
                        "Upload error: {:?}",
                        err
                    )));
                    // ignoring all errors
                    handles = remaining;
                }
            }
            if !assets.is_empty() {
                // if we are half way through, let spawn more transactions
                if (PARALLEL_LIMIT - handles.len()) > (PARALLEL_LIMIT / 2) {
                    // syncs cache (checkpoint)
                    cache.sync_file()?;
                    for task in assets.drain(0..cmp::min(assets.len(), PARALLEL_LIMIT / 2)) {
                        handles.push(self.upload_asset(task));
                    }
                }
            }
        }

        if errors.is_empty() && !assets.is_empty() {
            progress.abandon_with_message(format!("{}", style("Upload aborted ").red().bold()));
            return Err(
                UploadError::SendDataFailed("Not all files were uploaded.".to_string()).into(),
            );
        }

        Ok(errors)
    }
}

/// Returns a new uploader trait object based on the configuration `uploadMethod`.
///
/// This function acts as a *factory* function for uploader objects.
///
pub async fn initialize(
    sugar_config: &SugarConfig,
    config_data: &ConfigData,
) -> Result<Box<dyn Uploader>> {
    Ok(match config_data.upload_method {
        UploadMethod::AWS => Box::new(AWSMethod::new(config_data).await?) as Box<dyn Uploader>,
        UploadMethod::Bundlr => {
            Box::new(BundlrMethod::new(sugar_config, config_data).await?) as Box<dyn Uploader>
        }
        UploadMethod::NftStorage => {
            Box::new(NftStorageMethod::new(config_data).await?) as Box<dyn Uploader>
        }
        UploadMethod::SHDW => {
            Box::new(SHDWMethod::new(sugar_config, config_data).await?) as Box<dyn Uploader>
        }
    })
}
