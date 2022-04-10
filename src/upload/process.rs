use async_trait::async_trait;
use console::style;
use std::collections::HashSet;

use crate::cache::{load_cache, Cache};
use crate::common::*;
use crate::config::{data::SugarConfig, get_config_data, UploadMethod};
use crate::upload::bundlr::BundlrHandler;
use crate::upload::*;
use crate::utils::*;

/// A trait for storage upload handlers.
#[async_trait]
pub trait UploadHandler {
    /// Upload the data to a (permanent) storage.
    async fn upload_data(
        &self,
        config: &SugarConfig,
        assets: &HashMap<usize, AssetPair>,
        cache: &mut Cache,
        indices: &[usize],
        data_type: DataType,
    ) -> Result<Vec<UploadError>>;
}

pub struct UploadArgs {
    pub assets_dir: String,
    pub config: String,
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
}

pub async fn process_upload(args: UploadArgs) -> Result<()> {
    // loading assets
    println!(
        "{} {}Loading assets",
        style("[1/4]").bold().dim(),
        ASSETS_EMOJI
    );

    let pb = spinner_with_style();
    pb.enable_steady_tick(120);
    pb.set_message("Reading files...");

    let asset_pairs = get_asset_pairs(&args.assets_dir)?;
    // creates/loads the cache
    let mut cache = load_cache(&args.cache, true).unwrap();
    // list of indices to upload
    // 0: media
    // 1: metadata
    let mut indices = (Vec::new(), Vec::new());

    for (index, pair) in &asset_pairs {
        match cache.items.0.get_mut(&index.to_string()) {
            Some(item) => {
                // has the media file changed?
                if !item.media_hash.eq(&pair.media_hash) || item.media_link.is_empty() {
                    // we replace the entire item to trigger the media and metadata upload
                    cache
                        .items
                        .0
                        .insert(index.to_string(), pair.clone().into_cache_item());
                    // we need to upload both media/metadata
                    indices.0.push(*index);
                    indices.1.push(*index);
                } else if !item.metadata_hash.eq(&pair.metadata_hash)
                    || item.metadata_link.is_empty()
                {
                    // triggers the metadata upload
                    item.metadata_hash = pair.metadata_hash.clone();
                    item.metadata_link = String::new();
                    item.on_chain = false;
                    // we need to upload metadata only
                    indices.1.push(*index);
                }
            }
            None => {
                cache
                    .items
                    .0
                    .insert(index.to_string(), pair.clone().into_cache_item());
                // we need to upload both media/metadata
                indices.0.push(*index);
                indices.1.push(*index);
            }
        }
    }

    pb.finish_and_clear();

    println!(
        "Found {} media/metadata pair(s), uploading files:",
        asset_pairs.len()
    );
    println!("+--------------------+");
    println!("| media     | {:>6} |", indices.0.len());
    println!("| metadata  | {:>6} |", indices.1.len());
    println!("+--------------------+");

    // this should never happen, since every time we update the media file we
    // need to update the metadata
    if indices.0.len() > indices.1.len() {
        return Err(anyhow!(format!(
            "There are more media files ({}) to upload than metadata ({})",
            indices.0.len(),
            indices.1.len(),
        )));
    }

    let need_upload = !indices.0.is_empty() || !indices.1.is_empty();

    // ready to upload data

    let sugar_config = sugar_setup(args.keypair, args.rpc_url)?;
    let config_data = get_config_data(&args.config)?;
    let mut errors = Vec::new();

    if need_upload {
        println!(
            "\n{} {}Initiliazing upload",
            style("[2/4]").bold().dim(),
            COMPUTER_EMOJI
        );

        let handler = match config_data.upload_method {
            UploadMethod::Bundlr => Box::new(
                BundlrHandler::initialize(&get_config_data(&args.config)?, &sugar_config)
                    .await
                    .unwrap(),
            ) as Box<dyn UploadHandler>,
            _ => {
                return Err(anyhow!(format!(
                    "Upload method '{}' currently unsupported!",
                    &config_data.upload_method.to_string()
                )))
            }
        };

        println!(
            "\n{} {}Uploading media files {}",
            style("[3/4]").bold().dim(),
            UPLOAD_EMOJI,
            if indices.0.is_empty() {
                "(skipping)"
            } else {
                ""
            }
        );

        if !indices.0.is_empty() {
            errors.extend(
                handler
                    .upload_data(
                        &sugar_config,
                        &asset_pairs,
                        &mut cache,
                        &indices.0,
                        DataType::Media,
                    )
                    .await?,
            );

            // updates the list of metadata indices since the media upload
            // might fail - removes any index that the media upload failed
            if !indices.1.is_empty() {
                for index in indices.0 {
                    let item = cache.items.0.get(&index.to_string()).unwrap();

                    if item.media_link.is_empty() {
                        // no media link, not ready for metadata upload
                        indices.1.retain(|&x| x != index);
                    }
                }
            }
        }

        println!(
            "\n{} {}Uploading metadata files {}",
            style("[4/4]").bold().dim(),
            UPLOAD_EMOJI,
            if indices.1.is_empty() {
                "(skipping)"
            } else {
                ""
            }
        );

        if !indices.1.is_empty() {
            errors.extend(
                handler
                    .upload_data(
                        &sugar_config,
                        &asset_pairs,
                        &mut cache,
                        &indices.1,
                        DataType::Metadata,
                    )
                    .await?,
            );
        }
    } else {
        println!("\n....no files need uploading, skipping remaining steps.");
    }

    // sanity check

    cache.sync_file()?;

    let mut count = 0;

    for (_index, item) in cache.items.0 {
        if !(item.media_link.is_empty() || item.metadata_link.is_empty()) {
            count += 1;
        }
    }

    println!(
        "\n{}",
        style(format!(
            "{}/{} media/metadata pair(s) uploaded.",
            count,
            asset_pairs.len()
        ))
        .bold()
    );

    if count != asset_pairs.len() {
        let message = if !errors.is_empty() {
            let mut message = String::new();
            message.push_str(&format!(
                "Failed to upload all files, {0} error(s) occurred:",
                errors.len()
            ));

            let mut unique = HashSet::new();

            for err in errors {
                unique.insert(err.to_string());
            }

            for u in unique {
                message.push_str("\n\t• ");
                message.push_str(&u);
            }

            message
        } else {
            "Incorrect number of media/metadata pairs".to_string()
        };

        return Err(UploadError::Incomplete(message).into());
    }

    Ok(())
}
