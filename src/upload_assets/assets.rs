use bundlr_sdk::{tags::Tag, Bundlr, BundlrTx, SolanaSigner};
use futures::future::select_all;
use glob::glob;
use indicatif::ProgressBar;
use regex::Regex;
use serde_json;
use std::{
    fs::{self, File, OpenOptions},
    sync::{Arc, Mutex},
};

use crate::cache::*;
use crate::common::*;
use crate::upload_assets::errors::*;
use crate::validate::format::Metadata;

pub struct UploadDataArgs<'a> {
    pub bundlr_client: Arc<Bundlr<SolanaSigner>>,
    pub assets_dir: &'a Path,
    pub extension_glob: &'a str,
    pub tags: Vec<Tag>,
    pub data_type: DataType,
}

#[derive(Debug, Clone)]
pub enum DataType {
    Media,
    Metadata,
}

#[derive(Debug, Clone)]
pub struct AssetPair {
    pub name: String,
    pub metadata: String,
    pub media: String,
    pub media_link: String,
    pub metadata_link: String,
}

impl AssetPair {
    pub fn into_cache_item(self) -> CacheItem {
        CacheItem {
            name: self.name,
            link: self.metadata_link,
            on_chain: false,
        }
    }
}

pub fn get_data_size(assets_dir: &Path, extension: &str) -> Result<u64> {
    let path = assets_dir
        .join(format!("*.{extension}"))
        .to_str()
        .unwrap()
        .to_string();
    let assets = glob(&path)?;

    let mut total_size = 0;

    for asset in assets {
        let asset_path = asset?;
        let size = std::fs::metadata(asset_path)?.len();
        total_size += size;
    }

    Ok(total_size)
}

pub async fn upload_data<'a>(
    args: UploadDataArgs<'a>,
    asset_pairs: &mut HashMap<usize, AssetPair>,
) -> Result<()> {
    let path = args.assets_dir.join(args.extension_glob);
    let pattern = path.to_str().ok_or_else(|| {
        UploadAssetsError::InvalidAssetsDirectory(args.assets_dir.to_str().unwrap().to_string())
    })?;

    let (paths, errors): (Vec<_>, Vec<_>) = glob(pattern)?.into_iter().partition(Result::is_ok);

    let paths: Vec<_> = paths.into_iter().map(Result::unwrap).collect();
    let path_errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();

    let file_open_errors: Arc<Mutex<Vec<FileOpenError>>> = Arc::new(Mutex::new(Vec::new()));
    let deserialize_errors: Arc<Mutex<Vec<DeserializeError>>> = Arc::new(Mutex::new(Vec::new()));

    let bundlr_client = args.bundlr_client;
    let mut handles = Vec::new();
    println!("Sending data: (Ctrl+C to abort)");
    let pb = ProgressBar::new(asset_pairs.len().try_into().unwrap());

    for path in paths {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let asset_id = file_name.split('.').next().unwrap().to_string();

        let bundlr_client = bundlr_client.clone();
        let data = fs::read(&path)?;

        let tx = bundlr_client.create_transaction_with_tags(data, args.tags.clone());

        let handle = tokio::spawn(async move {
            send_bundlr_tx(bundlr_client.clone(), asset_id.clone(), tx).await
        });
        handles.push(handle);
    }

    while !handles.is_empty() {
        match select_all(handles).await {
            (Ok(res), _index, remaining) => {
                let val = res?;
                let link = format!("https://arweave.net/{}", val.clone().1);
                let id = val.0.parse::<usize>()?;
                let asset = asset_pairs
                    .get_mut(&id)
                    .unwrap_or_else(|| panic!("Failed to get asset {val:?}"));
                match args.data_type {
                    DataType::Media => {
                        asset.media_link = link;
                    }
                    DataType::Metadata => {
                        asset.metadata_link = link;
                    }
                }
                handles = remaining;
                // updates the progress bar
                pb.inc(1);
            }
            (Err(_e), _index, remaining) => {
                // Ignoring all errors
                handles = remaining;
            }
        }
    }

    pb.finish_with_message("Media upload successfully");

    if !path_errors.is_empty() {
        error!("Path errors: {:?}", path_errors);
        return Err(ReadFilesError::PathErrors.into());
    }

    if !file_open_errors.lock().unwrap().is_empty() {
        error!("File open errors: {:?}", file_open_errors);
        return Err(ReadFilesError::FileOpenErrors.into());
    }

    if !deserialize_errors.lock().unwrap().is_empty() {
        error!("Deserialize errors: {:?}", deserialize_errors);
        return Err(ReadFilesError::DeserializeErrors.into());
    }

    for handle in handles {
        let _result = handle.await?;
    }

    Ok(())
}

async fn send_bundlr_tx(
    bundlr_client: Arc<Bundlr<SolanaSigner>>,
    asset_id: String,
    tx: BundlrTx,
) -> Result<(String, String)> {
    let response = bundlr_client.send_transaction(tx).await?;
    let id = response.get("id").unwrap().as_str().unwrap();

    Ok((asset_id, id.to_string()))
}

pub fn get_media_extension(assets_dir: &str) -> Result<String> {
    let entries = fs::read_dir(assets_dir)?;

    let re = Regex::new(r".+\d+\.(\w+[^json|JSON])$").expect("Failed to create regex.");

    for entry in entries {
        let path = entry?.path();
        if let Some(captures) = re.captures(path.to_str().unwrap()) {
            let extension = captures.get(1).unwrap().as_str();
            return Ok(extension.to_string());
        }
    }

    Err(UploadAssetsError::GetExtensionError.into())
}

pub fn count_files(assets_dir: &str) -> Result<usize> {
    let files = fs::read_dir(assets_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            !entry.file_name().to_str().unwrap().starts_with('.')
                && entry.metadata().unwrap().is_file()
        });
    Ok(files.count())
}

pub fn get_asset_pairs(assets_dir: &str) -> Result<HashMap<usize, AssetPair>> {
    // filters out directories and hidden files
    let num_files = count_files(assets_dir)?;
    println!("Found {num_files} files");
    let mut asset_pairs: HashMap<usize, AssetPair> = HashMap::new();

    // Number of files should be even.
    if num_files % 2 != 0 {
        return Err(UploadAssetsError::InvalidNumberOfFiles(num_files).into());
    }

    let extension = get_media_extension(assets_dir)?;

    println!("Extension: {extension}");

    // Iterate over asset pairs.
    for i in 0..(num_files / 2) {
        let metadata_file = PathBuf::from(assets_dir).join(format!("{i}.json"));
        let metadata_file = metadata_file.to_str().unwrap().to_string();
        let media_file = Path::new(assets_dir).join(format!("{i}.{extension}"));

        let m = File::open(&metadata_file)?;
        let metadata: Metadata = serde_json::from_reader(m)?;
        let name = metadata.name.clone();

        let asset_pair = AssetPair {
            name,
            metadata: metadata_file,
            media: media_file.to_str().unwrap().to_string(),
            media_link: String::from(""),
            metadata_link: String::from(""),
        };
        asset_pairs.insert(i, asset_pair);
    }

    Ok(asset_pairs)
}

pub fn insert_media_links(asset_pairs: &HashMap<usize, AssetPair>) -> Result<()> {
    for (_, asset_pair) in asset_pairs.iter() {
        let mut metadata: Metadata = {
            let m = OpenOptions::new().read(true).open(&asset_pair.metadata)?;
            serde_json::from_reader(&m)?
        };
        metadata.image = asset_pair.media_link.clone();
        metadata.properties.files[0].uri = asset_pair.media_link.clone();

        let mut m = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&asset_pair.metadata)?;
        serde_json::to_writer(&mut m, &metadata)?;
    }

    Ok(())
}
