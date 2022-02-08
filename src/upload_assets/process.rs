use bundlr_sdk::{tags::Tag, Bundlr, SolanaSigner};
use clap::crate_version;
use std::{fs::File, sync::Arc};

use crate::cache::Cache;
use crate::common::*;
use crate::upload_assets::*;

pub struct UploadAssetsArgs {
    pub assets_dir: String,
    pub config: String,
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
}

pub async fn process_upload_assets(args: UploadAssetsArgs) -> Result<()> {
    let sugar_config = sugar_setup(args.keypair, args.rpc_url)?;
    let http_client = reqwest::Client::new();
    let client = setup_client(&sugar_config)?;

    let pid = "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ"
        .parse()
        .expect("Failed to parse PID");
    let program = client.program(pid);

    //   Get keypair as base58 string for Bundlr.
    let keypair = bs58::encode(sugar_config.keypair.to_bytes()).into_string();
    let signer = SolanaSigner::from_base58(&keypair);

    let bundlr_client = Bundlr::new(
        "https://node1.bundlr.network".to_string(),
        "solana".to_string(),
        "sol".to_string(),
        signer,
    );

    let bundlr_client = Arc::new(bundlr_client);

    let extension = get_media_extension(&args.assets_dir)?;

    println!("Extension: {extension}");

    let total_image_size = get_data_size(Path::new(&args.assets_dir), &extension)?;

    println!("Total image size: {}", total_image_size);

    let media_lamports_fee = get_bundlr_fee(&http_client, total_image_size).await?;

    let address = sugar_config.keypair.pubkey().to_string();
    let balance = get_bundlr_balance(&http_client, &address).await?;
    println!("Bundlr balance: {}", balance);

    let bundlr_address = get_bundlr_solana_address(&http_client).await?;
    let bundlr_pubkey = Pubkey::from_str(&bundlr_address)?;

    let _response = fund_bundlr_address(
        &program,
        &http_client,
        bundlr_pubkey,
        &sugar_config.keypair,
        media_lamports_fee,
    )
    .await?;

    let balance = get_bundlr_balance(&http_client, &address).await?;

    if !(balance > 0) {
        panic!("Failed to fund Bundlr account");
    }

    let sugar_tag = Tag::new("App-Name".into(), format!("Sugar {}", crate_version!()));
    let media_tag = Tag::new("Content-Type".into(), format!("image/{extension}"));
    let metadata_tag = Tag::new("Content-Type".into(), "application/json".to_string());

    let media_extension_glob = &format!("*.{extension}");
    let metadata_extension_glob = &format!("*.json");

    let mut asset_pairs = get_asset_pairs(&args.assets_dir)?;

    println!("retrieved asset pairs");

    let upload_media_args = UploadDataArgs {
        // keypair: sugar_config.keypair,
        bundlr_client: bundlr_client.clone(),
        assets_dir: Path::new(&args.assets_dir),
        extension_glob: media_extension_glob,
        tags: vec![sugar_tag.clone(), media_tag],
        data_type: DataType::Media,
    };
    upload_data(upload_media_args, &mut asset_pairs).await?;

    println!("{:?}", asset_pairs);
    insert_media_links(&asset_pairs)?;

    let total_metadata_size = get_data_size(Path::new(&args.assets_dir), "json")?;
    let metadata_lamports_fee = get_bundlr_fee(&http_client, total_metadata_size).await?;

    let _response = fund_bundlr_address(
        &program,
        &http_client,
        bundlr_pubkey,
        &sugar_config.keypair,
        metadata_lamports_fee,
    )
    .await?;

    let upload_metadata_args = UploadDataArgs {
        bundlr_client: bundlr_client.clone(),
        assets_dir: Path::new(&args.assets_dir),
        extension_glob: metadata_extension_glob,
        tags: vec![sugar_tag, metadata_tag],
        data_type: DataType::Metadata,
    };
    upload_data(upload_metadata_args, &mut asset_pairs).await?;

    let cache_file_path = Path::new(&args.cache);
    let mut cache: Cache = if cache_file_path.exists() {
        println!("Cache file exists, loading...");
        let file = File::open(cache_file_path)?;
        serde_json::from_reader(file)?
    } else {
        println!("Cache file does not exist, creating...");
        Cache::new()
    };

    let mut items = IndexMap::new();

    for (key, value) in asset_pairs {
        items.insert(key.to_string(), value.into_cache_item());
    }
    cache.items.0 = items;

    cache.write_to_file(cache_file_path)?;

    Ok(())
}
