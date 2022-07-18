use std::{
    borrow::Borrow,
    collections::HashSet,
    ffi::OsStr,
    fmt::Write as _,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use console::style;

use crate::{
    cache::{load_cache, Cache},
    common::*,
    config::{get_config_data, SugarConfig},
    upload::*,
    utils::*,
    validate::format::Metadata,
};

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
    let mut indices = AssetType {
        image: Vec::new(),
        metadata: Vec::new(),
        animation: Vec::new(),
    };

    for (index, pair) in &asset_pairs {
        match cache.items.get_mut(&index.to_string()) {
            Some(item) => {
                let image_changed =
                    !item.image_hash.eq(&pair.image_hash) || item.image_link.is_empty();

                let animation_changed = !item.animation_hash.eq(&pair.animation_hash)
                    || (item.animation_link.is_none() && pair.animation.is_some());

                let metadata_changed =
                    !item.metadata_hash.eq(&pair.metadata_hash) || item.metadata_link.is_empty();

                if image_changed {
                    // triggers the image upload
                    item.image_hash = pair.image_hash.clone();
                    item.image_link = String::new();
                    indices.image.push(*index);
                }

                if animation_changed {
                    // triggers the animation upload
                    item.animation_hash = pair.animation_hash.clone();
                    item.animation_link = None;
                    indices.animation.push(*index);
                }

                if metadata_changed || image_changed || animation_changed {
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
                // and we might need to upload the animation
                if pair.animation.is_some() {
                    indices.animation.push(*index);
                }
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
                // seller-fee-basis-points check, but only if the asset actually has the value
                if let Some(seller_fee_basis_points) = metadata.seller_fee_basis_points {
                    if config_data.seller_fee_basis_points != seller_fee_basis_points {
                        return Err(UploadError::MismatchValue(
                            "seller_fee_basis_points".to_string(),
                            pair.metadata.clone(),
                            config_data.seller_fee_basis_points.to_string(),
                            seller_fee_basis_points.to_string(),
                        )
                        .into());
                    }
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
        "Found {} asset pair(s), uploading files:",
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
            style(if indices.animation.is_empty() {
                "[2/4]"
            } else {
                "[2/5]"
            })
            .bold()
            .dim(),
            COMPUTER_EMOJI
        );

        let pb = spinner_with_style();
        pb.set_message("Connecting...");

        let storage = initialize(&sugar_config, &config_data).await?;

        pb.finish_with_message("Connected");

        storage
            .prepare(
                &sugar_config,
                &asset_pairs,
                vec![
                    (DataType::Image, &indices.image),
                    (DataType::Animation, &indices.animation),
                    (DataType::Metadata, &indices.metadata),
                ],
            )
            .await?;

        // clear the interruption handler value ahead of the upload
        args.interrupted.store(false, Ordering::SeqCst);

        println!(
            "\n{} {}Uploading image files {}",
            style(if indices.animation.is_empty() {
                "[3/4]"
            } else {
                "[3/5]"
            })
            .bold()
            .dim(),
            UPLOAD_EMOJI,
            if indices.image.is_empty() {
                "(skipping)"
            } else {
                ""
            }
        );

        if !indices.image.is_empty() {
            errors.extend(
                upload_data(
                    &sugar_config,
                    &asset_pairs,
                    &mut cache,
                    &indices.image,
                    DataType::Image,
                    storage.borrow(),
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
                upload_data(
                    &sugar_config,
                    &asset_pairs,
                    &mut cache,
                    &indices.animation,
                    DataType::Animation,
                    storage.borrow(),
                    args.interrupted.clone(),
                )
                .await?,
            );

            // updates the list of metadata indices since the image upload
            // might fail - removes any index that the animation upload failed
            if !indices.metadata.is_empty() {
                for index in indices.animation.clone() {
                    let item = cache.items.get(&index.to_string()).unwrap();

                    if item.animation_link.is_none() {
                        // no animation link, not ready for metadata upload
                        indices.metadata.retain(|&x| x != index);
                    }
                }
            }
        }

        println!(
            "\n{} {}Uploading metadata files {}",
            style(if indices.animation.is_empty() {
                "[4/4]"
            } else {
                "[5/5]"
            })
            .bold()
            .dim(),
            UPLOAD_EMOJI,
            if indices.metadata.is_empty() {
                "(skipping)"
            } else {
                ""
            }
        );

        if !indices.metadata.is_empty() {
            errors.extend(
                upload_data(
                    &sugar_config,
                    &asset_pairs,
                    &mut cache,
                    &indices.metadata,
                    DataType::Metadata,
                    storage.borrow(),
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
        style(format!(
            "{}/{} asset pair(s) uploaded.",
            count,
            asset_pairs.len()
        ))
        .bold()
    );

    if count != asset_pairs.len() {
        let message = if !errors.is_empty() {
            let mut message = String::new();
            write!(
                message,
                "Failed to upload all files, {0} error(s) occurred:",
                errors.len()
            )?;

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
            "Not all files were uploaded.".to_string()
        };

        return Err(UploadError::Incomplete(message).into());
    }

    Ok(())
}

/// Upload the data to the selected storage.
async fn upload_data(
    sugar_config: &SugarConfig,
    asset_pairs: &HashMap<isize, AssetPair>,
    cache: &mut Cache,
    indices: &[isize],
    data_type: DataType,
    uploader: &dyn Uploader,
    interrupted: Arc<AtomicBool>,
) -> Result<Vec<UploadError>> {
    let mut extension = HashSet::with_capacity(1);
    let mut paths = Vec::new();

    for index in indices {
        let item = match asset_pairs.get(index) {
            Some(asset_index) => asset_index,
            None => return Err(anyhow::anyhow!("Failed to get asset at index {}", index)),
        };
        // chooses the file path based on the data type
        let file_path = match data_type {
            DataType::Image => item.image.clone(),
            DataType::Metadata => item.metadata.clone(),
            DataType::Animation => {
                if let Some(animation) = item.animation.clone() {
                    animation
                } else {
                    return Err(anyhow::anyhow!(
                        "Missing animation path for asset at index {}",
                        index
                    ));
                }
            }
        };

        let path = Path::new(&file_path);
        let ext = path
            .extension()
            .and_then(OsStr::to_str)
            .expect("Failed to convert extension from unicode");
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

    // uploading data

    println!("\nSending data: (Ctrl+C to abort)");

    let pb = progress_bar_with_style(paths.len() as u64);

    let mut assets = Vec::new();

    for file_path in paths {
        // path to the media/metadata file
        let path = Path::new(&file_path);
        let file_name = String::from(
            path.file_name()
                .and_then(OsStr::to_str)
                .expect("Filed to get file name."),
        );
        let (asset_id, cache_item) = get_cache_item(path, cache)?;

        let content = match data_type {
            // replaces the media link without modifying the original file to avoid
            // changing the hash of the metadata file
            DataType::Metadata => get_updated_metadata(
                &file_path,
                &cache_item.image_link,
                &cache_item.animation_link,
            )?,
            _ => file_path.clone(),
        };

        assets.push(AssetInfo {
            asset_id: asset_id.to_string(),
            name: file_name,
            content,
            data_type: data_type.clone(),
            content_type: content_type.clone(),
        });
    }

    let errors = uploader
        .upload(
            sugar_config,
            cache,
            data_type,
            &mut assets,
            &pb,
            interrupted,
        )
        .await?;

    if !errors.is_empty() {
        pb.abandon_with_message(format!("{}", style("Upload failed ").red().bold()));
    } else {
        pb.finish_with_message(format!("{}", style("Upload successful ").green().bold()));
    }

    // makes sure the cache file is updated
    cache.sync_file()?;

    Ok(errors)
}
