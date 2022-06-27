use async_trait::async_trait;
use console::style;
use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::cache::{load_cache, Cache};
use crate::common::*;
use crate::config::{data::SugarConfig, get_config_data, UploadMethod};
use crate::upload::bundlr::BundlrHandler;
use crate::upload::*;
use crate::utils::*;
use crate::validate::format::Metadata;

/// A trait for storage upload handlers.
#[async_trait]
pub trait UploadHandler {
    /// Prepares the upload of the specified image/metadata files.
    async fn prepare(
        &self,
        sugar_config: &SugarConfig,
        assets: &HashMap<isize, AssetPair>,
        image_indices: &[isize],
        metadata_indices: &[isize],
        animation_indices: &[isize],
    ) -> Result<()>;

    /// Upload the data to a (permanent) storage.
    async fn upload_data(
        &self,
        sugar_config: &SugarConfig,
        assets: &HashMap<isize, AssetPair>,
        cache: &mut Cache,
        indices: &[isize],
        data_type: DataType,
        interrupted: Arc<AtomicBool>,
    ) -> Result<Vec<UploadError>>;
}

pub struct UploadArgs {
    pub assets_dir: String,
    pub config: String,
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub interrupted: Arc<AtomicBool>,
}

pub struct AssetType {
    pub image: Vec<isize>,
    pub metadata: Vec<isize>,
    pub animation: Vec<isize>,
}

pub async fn process_upload(args: UploadArgs) -> Result<()> {
    let sugar_config = sugar_setup(args.keypair, args.rpc_url)?;
    let config_data = get_config_data(&args.config)?;

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
    let mut cache = load_cache(&args.cache, true)?;
    if asset_pairs.get(&-1).is_none() {
        cache.items.remove("-1");
    }

    // list of indices to upload
    // 0: image
    // 1: metadata
    let mut indices = AssetType {
        image: Vec::new(),
        metadata: Vec::new(),
        animation: Vec::new(),
    };

    for (index, pair) in &asset_pairs {
        match cache.items.get_mut(&index.to_string()) {
            Some(item) => {
                // determining animation condition
                let animation_condition =
                    if item.animation_hash.is_some() && item.animation_link.as_ref().is_some() {
                        !item.animation_hash.eq(&pair.animation_hash)
                            || item.animation_link.as_ref().unwrap().is_empty()
                    } else {
                        false
                    };

                // has the image file changed?
                if !&item.image_hash.eq(&pair.image_hash) || item.image_link.is_empty() {
                    // we replace the entire item to trigger the image and metadata upload
                    let item_clone = item.clone();
                    cache
                        .items
                        .insert(index.to_string(), pair.clone().into_cache_item());
                    // we need to upload both image/metadata
                    indices.image.push(*index);
                    indices.metadata.push(*index);

                    if item_clone.animation_hash.is_some() || item_clone.animation_link.is_some() {
                        indices.animation.push(*index);
                    }
                } else if animation_condition {
                    // we replace the entire item to trigger the image and metadata upload
                    cache
                        .items
                        .insert(index.to_string(), pair.clone().into_cache_item());
                    // we need to upload both image/metadata
                    indices.animation.push(*index);
                    indices.image.push(*index);
                    indices.metadata.push(*index);
                } else if !item.metadata_hash.eq(&pair.metadata_hash)
                    || item.metadata_link.is_empty()
                {
                    // triggers the metadata upload
                    item.metadata_hash = pair.metadata_hash.clone();
                    item.metadata_link = String::new();
                    item.on_chain = false;
                    // we need to upload metadata only
                    indices.metadata.push(*index);
                }
            }
            None => {
                cache
                    .items
                    .insert(index.to_string(), pair.clone().into_cache_item());
                // we need to upload both image/metadata
                indices.image.push(*index);
                indices.metadata.push(*index);

                if pair.animation_hash.clone().is_some() {
                    indices.animation.push(*index);
                };
            }
        }
        // sanity check: verifies that both symbol and seller-fee-basis-points are the
        // same as the ones in the config file
        let f = File::open(Path::new(&pair.metadata))?;
        match serde_json::from_reader(f) {
            Ok(metadata) => {
                let metadata: Metadata = metadata;
                // symbol check
                if config_data.symbol.ne(&metadata.symbol) {
                    return Err(UploadError::MismatchValue(
                        "symbol".to_string(),
                        pair.metadata.clone(),
                        config_data.symbol,
                        metadata.symbol,
                    )
                    .into());
                }
                // seller-fee-basis-points check
                if config_data.seller_fee_basis_points != metadata.seller_fee_basis_points {
                    return Err(UploadError::MismatchValue(
                        "seller_fee_basis_points".to_string(),
                        pair.metadata.clone(),
                        config_data.seller_fee_basis_points.to_string(),
                        metadata.seller_fee_basis_points.to_string(),
                    )
                    .into());
                }
            }
            Err(err) => {
                let error = anyhow!("Error parsing metadata ({}): {}", pair.metadata, err);
                error!("{:?}", error);
                return Err(error);
            }
        }
    }

    pb.finish_and_clear();

    println!(
        "Found {} image/metadata pair(s), uploading files:",
        asset_pairs.len()
    );
    println!("+--------------------+");
    println!("| images    | {:>6} |", indices.image.len());
    println!("| metadata  | {:>6} |", indices.metadata.len());
    if !indices.animation.is_empty() {
        println!("| animation | {:>6} |", indices.animation.len());
    }
    println!("+--------------------+");

    // this should never happen, since every time we update the image file we
    // need to update the metadata
    if indices.image.len() > indices.metadata.len() {
        return Err(anyhow!(format!(
            "There are more image files ({}) to upload than metadata ({})",
            indices.image.len(),
            indices.metadata.len(),
        )));
    }

    let need_upload =
        !indices.image.is_empty() || !indices.metadata.is_empty() || !indices.animation.is_empty();

    // ready to upload data

    let mut errors = Vec::new();

    if need_upload {
        println!(
            "\n{} {}Initializing upload",
            if !indices.animation.is_empty() {
                style("[2/5]").bold().dim()
            } else {
                style("[2/4]").bold().dim()
            },
            COMPUTER_EMOJI
        );

        let pb = spinner_with_style();
        pb.set_message("Connecting...");

        let handler = match config_data.upload_method {
            UploadMethod::Bundlr => Box::new(
                BundlrHandler::initialize(&get_config_data(&args.config)?, &sugar_config).await?,
            ) as Box<dyn UploadHandler>,
            UploadMethod::AWS => {
                Box::new(AWSHandler::initialize(&get_config_data(&args.config)?).await?)
                    as Box<dyn UploadHandler>
            }
            UploadMethod::NftStorage => {
                Box::new(NftStorageHandler::initialize(&get_config_data(&args.config)?).await?)
                    as Box<dyn UploadHandler>
            }
        };

        pb.finish_with_message("Connected");

        handler
            .prepare(
                &sugar_config,
                &asset_pairs,
                &indices.image,
                &indices.metadata,
                &indices.animation,
            )
            .await?;

        // clear the interruption handler value ahead of the upload
        args.interrupted.store(false, Ordering::SeqCst);

        println!(
            "\n{} {}Uploading image files {}",
            if !indices.animation.is_empty() {
                style("[3/5]").bold().dim()
            } else {
                style("[3/4]").bold().dim()
            },
            UPLOAD_EMOJI,
            if indices.image.is_empty() {
                "(skipping)"
            } else {
                ""
            }
        );

        if !indices.image.is_empty() {
            errors.extend(
                handler
                    .upload_data(
                        &sugar_config,
                        &asset_pairs,
                        &mut cache,
                        &indices.image,
                        DataType::Image,
                        args.interrupted.clone(),
                    )
                    .await?,
            );

            // updates the list of metadata indices since the image upload
            // might fail - removes any index that the image upload failed
            if !indices.metadata.is_empty() {
                for index in indices.image {
                    let item = cache.items.get(&index.to_string()).unwrap();

                    if item.image_link.is_empty() {
                        // no image link, not ready for metadata upload
                        indices.metadata.retain(|&x| x != index);
                    }
                }
            }
        }

        if !indices.animation.is_empty() {
            println!(
                "\n{} {}Uploading animation files {}",
                style("[4/5]").bold().dim(),
                UPLOAD_EMOJI,
                if indices.animation.is_empty() {
                    "(skipping)"
                } else {
                    ""
                }
            );
        }

        if !indices.animation.is_empty() {
            errors.extend(
                handler
                    .upload_data(
                        &sugar_config,
                        &asset_pairs,
                        &mut cache,
                        &indices.animation,
                        DataType::Animation,
                        args.interrupted.clone(),
                    )
                    .await?,
            );

            // updates the list of metadata indices since the image upload
            // might fail - removes any index that the image upload failed
            if !indices.metadata.is_empty() {
                for index in indices.animation.clone() {
                    let item = cache.items.get(&index.to_string()).unwrap();

                    if item.animation_link.as_ref().unwrap().is_empty() {
                        // no image link, not ready for metadata upload
                        indices.metadata.retain(|&x| x != index);
                    }
                }
            }
        }

        println!(
            "\n{} {}Uploading metadata files {}",
            if !indices.animation.is_empty() {
                style("[5/5]").bold().dim()
            } else {
                style("[4/4]").bold().dim()
            },
            UPLOAD_EMOJI,
            if indices.metadata.is_empty() {
                "(skipping)"
            } else {
                ""
            }
        );

        if !indices.metadata.is_empty() {
            errors.extend(
                handler
                    .upload_data(
                        &sugar_config,
                        &asset_pairs,
                        &mut cache,
                        &indices.metadata,
                        DataType::Metadata,
                        args.interrupted.clone(),
                    )
                    .await?,
            );
        }
    } else {
        println!("\n....no files need uploading, skipping remaining steps.");
    }

    // sanity check

    cache.items.sort_keys();
    cache.sync_file()?;

    let mut count = 0;

    for (_index, item) in cache.items.0 {
        let has_animation = if let Some(animation_link) = item.animation_link {
            animation_link.is_empty()
        } else {
            false
        };

        if !(item.image_link.is_empty() || item.metadata_link.is_empty() || has_animation) {
            count += 1;
        }
    }

    println!(
        "\n{}",
        if !indices.animation.is_empty() {
            style(format!(
                "{}/{} image/animation/metadata pair(s) uploaded.",
                count,
                asset_pairs.len()
            ))
            .bold()
        } else {
            style(format!(
                "{}/{} image/metadata pair(s) uploaded.",
                count,
                asset_pairs.len()
            ))
            .bold()
        }
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
                message.push_str(&style("\n=> ").dim().to_string());
                message.push_str(&u);
            }

            message
        } else {
            "Incorrect number of image/metadata pairs".to_string()
        };

        return Err(UploadError::Incomplete(message).into());
    }

    Ok(())
}
