use std::{
    ffi::OsStr,
    fs::{self, DirEntry, File, OpenOptions},
    io::{BufReader, Read},
    sync::Arc,
};

use bundlr_sdk::{tags::Tag, Bundlr, Ed25519Signer as SolanaSigner};
use data_encoding::HEXLOWER;
use glob::glob;
use regex::{Regex, RegexBuilder};
use ring::digest::{Context, SHA256};
use serde::Serialize;
use serde_json;

use crate::{common::*, validate::format::Metadata};

pub struct UploadDataArgs<'a> {
    pub bundlr_client: Arc<Bundlr<SolanaSigner>>,
    pub assets_dir: &'a Path,
    pub extension_glob: &'a str,
    pub tags: Vec<Tag>,
    pub data_type: DataType,
}

#[derive(Debug, Clone)]
pub enum DataType {
    Image,
    Metadata,
    Animation,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssetPair {
    pub name: String,
    pub metadata: String,
    pub metadata_hash: String,
    pub image: String,
    pub image_hash: String,
    pub animation: Option<String>,
    pub animation_hash: Option<String>,
}

impl AssetPair {
    pub fn into_cache_item(self) -> CacheItem {
        CacheItem {
            name: self.name,
            image_hash: self.image_hash,
            image_link: String::new(),
            metadata_hash: self.metadata_hash,
            metadata_link: String::new(),
            on_chain: false,
            animation_hash: self.animation_hash,
            animation_link: self.animation,
        }
    }
}

pub fn get_cache_item<'a>(path: &Path, cache: &'a mut Cache) -> Result<(String, &'a CacheItem)> {
    let file_stem = String::from(
        path.file_stem()
            .and_then(OsStr::to_str)
            .expect("Failed to get convert path file ext to valid unicode."),
    );

    // id of the asset (to be used to update the cache link)
    let asset_id = if file_stem == "collection" {
        String::from("-1")
    } else {
        file_stem
    };

    let cache_item: &CacheItem = cache
        .items
        .get(&asset_id)
        .ok_or_else(|| anyhow!("Failed to get config item at index '{}'", asset_id))?;

    Ok((asset_id, cache_item))
}

pub fn get_data_size(assets_dir: &Path, extension: &str) -> Result<u64> {
    let path = assets_dir
        .join(format!("*.{extension}"))
        .to_str()
        .expect("Failed to convert asset directory path from unicode.")
        .to_string();

    let assets = glob(&path)?;

    let mut total_size = 0;

    for asset in assets {
        let asset_path = asset?;
        let size = fs::metadata(asset_path)?.len();
        total_size += size;
    }

    Ok(total_size)
}

pub fn list_files(assets_dir: &str, include_collection: bool) -> Result<Vec<DirEntry>> {
    let files = fs::read_dir(assets_dir)
        .map_err(|_| anyhow!("Failed to read assets directory"))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let is_file = entry
                .metadata()
                .expect("Failed to retrieve metadata from file")
                .is_file();

            let path = entry.path();
            let file_stem = path
                .file_stem()
                .unwrap_or_default()
                .to_str()
                .expect("Failed to convert file name to valid unicode.");

            let is_collection = include_collection && file_stem == "collection";
            let is_numeric = file_stem.chars().all(|c| c.is_ascii_digit());

            is_file && (is_numeric || is_collection)
        });

    Ok(files.collect())
}

pub fn get_asset_pairs(assets_dir: &str) -> Result<HashMap<isize, AssetPair>> {
    // filters out directories and hidden files
    let filtered_files = list_files(assets_dir, true)?;

    let paths = filtered_files
        .into_iter()
        .map(|entry| {
            let file_name_as_string =
                String::from(entry.path().file_name().unwrap().to_str().unwrap());
            file_name_as_string
        })
        .collect::<Vec<String>>();

    let mut asset_pairs: HashMap<isize, AssetPair> = HashMap::new();

    let paths_ref = &paths;

    let animation_exists_regex =
        Regex::new("^(.+)\\.((mp4)|(mov)|(webm))$").expect("Failed to create regex.");

    // since there doesn't have to be video for each image/json pair, need to get rid of
    // invalid file names before entering metadata filename loop
    for x in paths_ref {
        if let Some(captures) = animation_exists_regex.captures(x) {
            if &captures[1] != "collection" && captures[1].parse::<usize>().is_err() {
                let error = anyhow!("Couldn't parse filename '{}' to a valid index number.", x);
                error!("{:?}", error);
                return Err(error);
            }
        }
    }

    let metadata_filenames = paths_ref
        .clone()
        .into_iter()
        .filter(|p| p.to_lowercase().ends_with(".json"))
        .collect::<Vec<String>>();

    ensure_sequential_files(metadata_filenames.clone())?;

    for metadata_filename in metadata_filenames {
        let i = metadata_filename.split('.').next().unwrap();
        let is_collection_index = i == "collection";

        let index: isize = if is_collection_index {
            -1
        } else if let Ok(index) = i.parse::<isize>() {
            index
        } else {
            let error = anyhow!(
                "Couldn't parse filename '{}' to a valid index number.",
                metadata_filename
            );
            error!("{:?}", error);
            return Err(error);
        };

        let img_pattern = format!("^{}\\.((jpg)|(gif)|(png))$", i);

        let img_regex = RegexBuilder::new(&img_pattern)
            .case_insensitive(true)
            .build()
            .expect("Failed to create regex.");

        let img_filenames = paths_ref
            .clone()
            .into_iter()
            .filter(|p| img_regex.is_match(p))
            .collect::<Vec<String>>();

        let img_filename = if img_filenames.is_empty() {
            let error = if is_collection_index {
                anyhow!("Couldn't find the collection image filename.")
            } else {
                anyhow!(
                    "Couldn't find an image filename at index {}.",
                    i.parse::<isize>().unwrap()
                )
            };
            error!("{:?}", error);
            return Err(error);
        } else {
            &img_filenames[0]
        };

        // need a similar check for animation as above, this one checking if there is animation
        // on specific index

        let animation_pattern = format!("^{}\\.((mp4)|(mov)|(webm))$", i);
        let animation_regex = RegexBuilder::new(&animation_pattern)
            .case_insensitive(true)
            .build()
            .expect("Failed to create regex.");

        let animation_filenames = paths_ref
            .clone()
            .into_iter()
            .filter(|p| animation_regex.is_match(p))
            .collect::<Vec<String>>();

        let metadata_filepath = Path::new(assets_dir)
            .join(&metadata_filename)
            .to_str()
            .expect("Failed to convert metadata path from unicode.")
            .to_string();

        let m = File::open(&metadata_filepath)?;
        let metadata: Metadata = serde_json::from_reader(m).map_err(|e| {
            anyhow!("Failed to read metadata file '{metadata_filepath}' with error: {e}")
        })?;
        let name = metadata.name.clone();

        let img_filepath = Path::new(assets_dir)
            .join(img_filename)
            .to_str()
            .expect("Failed to convert image path from unicode.")
            .to_string();

        let animation_filename = if !animation_filenames.is_empty() {
            let animation_filepath = Path::new(assets_dir)
                .join(&animation_filenames[0])
                .to_str()
                .expect("Failed to convert image path from unicode.")
                .to_string();

            Some(animation_filepath)
        } else {
            None
        };

        let animation_hash = if let Some(animation_file) = &animation_filename {
            let encoded_filename = encode(animation_file)?;
            Some(encoded_filename)
        } else {
            None
        };

        let asset_pair = AssetPair {
            name,
            metadata: metadata_filepath.clone(),
            metadata_hash: encode(&metadata_filepath)?,
            image: img_filepath.clone(),
            image_hash: encode(&img_filepath)?,
            animation_hash,
            animation: animation_filename,
        };

        asset_pairs.insert(index, asset_pair);
    }

    Ok(asset_pairs)
}

pub fn encode(file: &str) -> Result<String> {
    let input = File::open(file)?;
    let mut reader = BufReader::new(input);
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(HEXLOWER.encode(context.finish().as_ref()))
}

fn ensure_sequential_files(metadata_filenames: Vec<String>) -> Result<()> {
    let mut metadata_indices = metadata_filenames
        .into_iter()
        .filter(|f| !f.starts_with("collection"))
        .map(|f| {
            f.split('.')
                .next()
                .unwrap()
                .to_string()
                .parse::<usize>()
                .map_err(|_| {
                    anyhow!(
                        "Couldn't parse metadata filename '{}' to a valid index number.",
                        f
                    )
                })
        })
        .collect::<Result<Vec<usize>>>()?;
    metadata_indices.sort_unstable();

    metadata_indices
        .into_iter()
        .enumerate()
        .try_for_each(|(i, file_index)| {
            if i != file_index {
                Err(anyhow!("Missing metadata file '{}.json'", i))
            } else {
                Ok(())
            }
        })
}

pub fn get_updated_metadata(
    metadata_file: &str,
    image_link: &str,
    animation_link: &Option<String>,
) -> Result<String> {
    let mut metadata: Metadata = {
        let m = OpenOptions::new()
            .read(true)
            .open(metadata_file)
            .map_err(|e| {
                anyhow!("Failed to read metadata file '{metadata_file}' with error: {e}")
            })?;
        serde_json::from_reader(&m)?
    };

    for file in &mut metadata.properties.files {
        if file.uri.eq(&metadata.image) {
            file.uri = image_link.to_string();
        }
        if let Some(ref animation_link) = animation_link {
            if let Some(ref animation_url) = metadata.animation_url {
                if file.uri.eq(animation_url) {
                    file.uri = animation_link.to_string();
                }
            }
        }
    }

    metadata.image = image_link.to_string();
    metadata.animation_url = animation_link.clone();

    Ok(serde_json::to_string(&metadata).unwrap())
}
