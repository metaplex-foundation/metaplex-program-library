use bundlr_sdk::{tags::Tag, Bundlr, SolanaSigner};
use data_encoding::HEXLOWER;
use glob::glob;
use regex::{Regex, RegexBuilder};
use ring::digest::{Context, SHA256};
use serde_json;
use std::{
    fs::{self, DirEntry, File, OpenOptions},
    io::{BufReader, Read},
    sync::Arc,
};

use crate::common::*;
use crate::upload::errors::*;
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
    Img,
    Metadata,
    Movie,
}

#[derive(Debug, Clone)]
pub struct AssetPair {
    pub name: String,
    pub metadata: String,
    pub metadata_hash: String,
    pub media: String,
    pub media_hash: String,
}

impl AssetPair {
    pub fn into_cache_item(self) -> CacheItem {
        CacheItem {
            name: self.name,
            media_hash: self.media_hash,
            media_link: String::new(),
            metadata_hash: self.metadata_hash,
            metadata_link: String::new(),
            on_chain: false,
        }
    }
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
        let size = std::fs::metadata(asset_path)?.len();
        total_size += size;
    }

    Ok(total_size)
}

// pub fn get_media_extension(filtered_files: Vec<DirEntry>, index: usize) -> Result<Vec<&str>> {
//     let files = Vec::new();

//     // let re = Regex::new(r".+\d+\.(\w+[^json|JSON])$").expect("Failed to create regex.");
//     let re = Regex::new(r"\.[^.]{1,}").expect("Failed to create regex.");

//     // for entry in filtered_files {
//     //     let path = entry?.path();
//     //     if let Some(captures) =
//     //         re.captures(path.to_str().expect("Failed to convert to valid unicode."))
//     //     {
//     //         let extension = captures.get(1).unwrap().as_str();
//     //         files.push(extension);
//     //         // return Ok(extension.to_string());
//     //     }
//     // }

//     if let Some(captures) = re.captures(filtered_files) {
//         println!("{:?}", captures);
//     };

//     Ok(files)
//     // Err(UploadError::GetExtensionError.into())
// }

pub fn count_files(assets_dir: &str) -> Result<Vec<DirEntry>> {
    let files = fs::read_dir(assets_dir)
        .map_err(|_| anyhow!("Failed to read assets directory"))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            !entry
                .file_name()
                .to_str()
                .expect("Failed to convert file name to valid unicode.")
                .starts_with('.')
                && entry
                    .metadata()
                    .expect("Failed to retrieve metadata from file")
                    .is_file()
        });

    Ok(files.collect())
}

pub fn get_asset_pairs(assets_dir: &str) -> Result<HashMap<usize, AssetPair>> {
    // filters out directories and hidden files
    let filtered_files = count_files(assets_dir)?;

    let paths = filtered_files
        .into_iter()
        .map(|entry| {
            let entry_path = entry.path();
            let file_name = entry_path.file_name().unwrap();
            let file_name_as_str = file_name.to_str().unwrap();
            let file_name_as_string = String::from(file_name_as_str);
            file_name_as_string
        })
        .collect::<Vec<String>>();

    let mut asset_pairs: HashMap<usize, AssetPair> = HashMap::new();

    // let last = &filtered_files[filtered_files.len() - 1]
    //     .path()
    //     .into_os_string()
    //     .into_string()
    //     .unwrap();

    // let mut v: Vec<&str> = last.split('/').collect();
    // v = v[1].split('.').collect();
    // let num_files = v[0].parse::<usize>().unwrap();

    // TODO: should we enforce that all files have the same extension?
    //let extension = "png";

    // todo: case sensitivity on extension
    let metadata_filenames = paths
        .into_iter()
        .filter(|p| p.ends_with(".json"))
        .collect::<Vec<String>>();

    for metadata_filename in metadata_filenames {
        // TODO: parse i here first to verify that is an integer

        let i = metadata_filename.split(".").nth(0).unwrap();
        let img_pattern = format!("^{}\\.((jpg)|(gif)|(png))$", i); //"^" + i.to_string() + "\\.[^.]+$";
        let img_regex = RegexBuilder::new(&img_pattern)
            .case_insensitive(true)
            .build()
            .expect("Failed to create regex.");
        let media_filenames = paths
            .into_iter()
            .filter(|p| img_regex.is_match(p))
            .collect::<Vec<String>>();
        let filename_of_media_file = media_filenames[0];
        let animation_pattern = format!("^{}\\.((mp4)|(mov)|(webm))$", i);
        let animation_regex = RegexBuilder::new(&animation_pattern)
            .case_insensitive(true)
            .build()
            .expect("Failed to create regex.");
        let animation_filenames = paths
            .iter()
            .filter(|p| animation_regex.is_match(p))
            .collect::<Vec<String>>();
        let filename_of_animation_file = if let Some(filename) = animation_filenames[0] {
            filename
        } else {
            None
        };

        let m = File::open(&metadata_filename)?;
        let metadata: Metadata = serde_json::from_reader(m)?;
        let name = metadata.name.clone();

        let metadata_filepath = PathBuf::new(assets_dir)
            .join(metadata_filename)
            .to_str()
            .expect("Failed to convert metadata path from unicode.")
            .to_string();
        let media_filepath = PathBuf::new(assets_dir)
            .join(media_filename)
            .to_str()
            .expect("Failed to convert media path from unicode.")
            .to_string();

        let asset_pair = AssetPair {
            name,
            metadata: metadata_filepath.clone(),
            metadata_hash: encode(&metadata_filepath)?,
            media: media_filepath.clone(),
            media_hash: encode(&media_filepath)?,
        };

        asset_pairs.insert(
            i.parse::<usize>().expect("Failed to parse filename number"),
            asset_pair,
        );
    }

    // todo: add error results for bogus stuff
    /*
    // iterate over asset pairs
    for i in 0..=1 {
        // let file = filtered_files[i];
        // let extensions = get_media_extension(assets_dir, index)

        let metadata_file = PathBuf::from(assets_dir)
            .join(format!("{i}.json"))
            .to_str()
            .expect("Failed to convert metadata path from unicode.")
            .to_string();

        let media_filename_extension = image_filename_extension_map.get(i);

        let media_file = Path::new(assets_dir)
            .join(format!("{i}.{extension}"))
            .to_str()
            .expect("Failed to convert media path from unicode.")
            .to_string();

        let m = File::open(&metadata_file)?;
        let metadata: Metadata = serde_json::from_reader(m)?;
        let name = metadata.name.clone();

        let asset_pair = AssetPair {
            name,
            metadata: metadata_file.clone(),
            metadata_hash: encode(&metadata_file)?,
            media: media_file.clone(),
            media_hash: encode(&media_file)?,
        };

        asset_pairs.insert(i, asset_pair);
    }*/

    Ok(asset_pairs)
}

fn encode(file: &str) -> Result<String> {
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

pub fn get_updated_metadata(metadata_file: &str, media_link: &str) -> Result<String> {
    let mut metadata: Metadata = {
        let m = OpenOptions::new().read(true).open(metadata_file)?;
        serde_json::from_reader(&m)?
    };

    metadata.image = media_link.to_string();
    metadata.properties.files[0].uri = media_link.to_string();

    Ok(serde_json::to_string(&metadata).unwrap())
}
