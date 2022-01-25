#![allow(unused)]
use anchor_client::solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    system_instruction, system_program, sysvar,
};
use anyhow::Result;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rand::rngs::OsRng;
use rayon::prelude::*;
use slog::*;
use std::{
    collections::HashMap,
    fs::File,
    path::Path,
    str::FromStr,
    sync::{Arc, Mutex},
};

use mpl_candy_machine::accounts as nft_accounts;
use mpl_candy_machine::instruction as nft_instruction;
use mpl_candy_machine::{CandyMachineData, ConfigLine, Creator as CandyCreator};

use crate::candy_machine::{uuid_from_pubkey, ConfigStatus};
use crate::config::{data::*, parser::get_config_data};
use crate::constants::*;
use crate::setup::{setup_client, sugar_setup};
use crate::validate::format::Metadata;

pub mod data;
pub use data::*;

pub fn process_upload(upload_args: UploadArgs) -> Result<()> {
    let sugar_config = sugar_setup(upload_args.logger, upload_args.keypair, upload_args.rpc_url)?;

    #[allow(unused)]
    let arloader_manifest = read_arloader_manifest(&upload_args.arloader_manifest)?;
    #[allow(unused)]
    let mut cache = Cache::new();
    #[allow(unused)]
    let mut config_lines = Vec::new();
    #[allow(unused)]
    let mut candy_pubkey = Pubkey::default();

    let cache_file_path = Path::new(&upload_args.cache);

    if cache_file_path.exists() {
        info!(sugar_config.logger, "Cache exists, loading...");
        let file = File::open(cache_file_path)?;
        cache = serde_json::from_reader(file)?;

        let candy_machine_address = &cache.program.candy_machine;

        // Empty string.
        if candy_machine_address.is_empty() {
            info!(
                sugar_config.logger,
                "Candy machine address is empty, creating new candy machine..."
            );
            let candy_keypair = Keypair::generate(&mut OsRng);
            candy_pubkey = candy_keypair.pubkey();

            let uuid = uuid_from_pubkey(&candy_pubkey);
            let config_data = get_config_data(&upload_args.config)?;
            let metadata = get_metadata_from_first_json(&upload_args.assets_dir)?;
            let candy_data = create_candy_machine_data(config_data, uuid, metadata)?;

            let sig = initialize_candy_machine(&sugar_config, &candy_keypair, candy_data)?;
            info!(
                sugar_config.logger,
                "Candy machine created with address: {}",
                &candy_pubkey.to_string()
            );

            cache.program = CacheProgram::new_from_cm(&candy_pubkey);
            cache.write_to_file(cache_file_path)?;
        } else {
            candy_pubkey = match Pubkey::from_str(candy_machine_address) {
                Ok(pubkey) => pubkey,
                Err(err) => {
                    error!(
                        sugar_config.logger,
                        "Invalid candy machine address in cache file: {}!", candy_machine_address
                    );
                    std::process::exit(1);
                }
            }
        };

        info!(sugar_config.logger, "Uploading config lines...");
        config_lines = generate_config_lines(&cache.items);
        let config_statuses = upload_config_lines(&sugar_config, config_lines, candy_pubkey)?;

        for status in config_statuses {
            let index: String = status.index.to_string();
            let mut item = cache.items.0.get_mut(&index).unwrap();
            item.on_chain = status.on_chain;
        }

        // Update cache with config statuses and cm data
        cache.write_to_file(cache_file_path)?;
    } else {
        info!(
            sugar_config.logger,
            "No cache file found at cache path: {:?}. Creating one.", cache_file_path
        );
        // let mut f = File::create("cache.json")?;
        // // Replace with Arweave upload logic
        // populate_cache_with_links(&mut cache, &arloader_manifest)?;
        // let c = serde_json::to_string(&cache)?;
        // f.write_all(c.as_bytes())?;
        // let candy_keypair = Keypair::generate(&mut OsRng);
        // candy_pubkey = candy_keypair.pubkey();

        // let config_lines: Vec<ConfigLine> = generate_config_lines(&cache.items);
        // let config_data = get_config_data(&upload_args.config)?;

        // let metadata = get_metadata_from_first_json(&upload_args.assets_dir)?;
        // let uuid = uuid_from_pubkey(&candy_pubkey);
        // let candy_data = create_candy_machine_data(config_data, uuid, metadata)?;
        // let sig = initialize_candy_machine(&sugar_config, &candy_keypair, candy_data)?;

        // // upload to Arweave

        // cache.program = CacheProgram::new_from_cm(&candy_pubkey);

        // // Todo: only upload ones with on_chain: false status
        // let config_statuses = upload_config_lines(&sugar_config, config_lines, candy_pubkey)?;

        // for status in config_statuses {
        //     let index: String = status.index.to_string();
        //     let mut item = cache.items.0.get_mut(&index).unwrap();
        //     item.on_chain = status.on_chain;
        // }

        // println!("{:?}", &cache);

        // // Update cache with config statuses and cm data
        // cache.write_to_file("cache.json")?;
    }

    Ok(())
}

fn get_metadata_from_first_json(assets_dir: &String) -> Result<Metadata> {
    let f = File::open(Path::new(assets_dir).join("0.json"))?;
    let metadata: Metadata = serde_json::from_reader(f)?;

    Ok(metadata)
}

fn create_candy_machine_data(
    config: ConfigData,
    uuid: String,
    metadata: Metadata,
) -> Result<CandyMachineData> {
    let go_live_date = Some(go_live_date_as_timestamp(&config.go_live_date)?);

    let end_settings = if let Some(settings) = config.end_settings {
        Some(settings.into_candy_format())
    } else {
        None
    };

    let whitelist_mint_settings = if let Some(settings) = config.whitelist_mint_settings {
        Some(settings.into_candy_format())
    } else {
        None
    };

    let hidden_settings = if let Some(settings) = config.hidden_settings {
        Some(settings.into_candy_format())
    } else {
        None
    };

    let gatekeeper = if let Some(gatekeeper) = config.gatekeeper {
        Some(gatekeeper.into_candy_format())
    } else {
        None
    };

    let mut creators: Vec<CandyCreator> = Vec::new();

    for creator in metadata.properties.creators {
        creators.push(creator.into_candy_format()?);
    }

    let data = CandyMachineData {
        uuid,
        price: price_as_lamports(config.price),
        symbol: String::default(),
        seller_fee_basis_points: u16::default(),
        max_supply: u64::default(),
        is_mutable: config.is_mutable,
        retain_authority: config.retain_authority,
        go_live_date,
        end_settings,
        creators,
        whitelist_mint_settings,
        hidden_settings,
        items_available: config.number,
        gatekeeper,
    };
    Ok(data)
}

fn populate_cache_with_links(
    cache: &mut Cache,
    arloader_manifest: &ArloaderManifest,
) -> Result<()> {
    let mut cache_items: CacheItems = CacheItems(HashMap::new());

    for (key, value) in &arloader_manifest.0 {
        let name = key
            .split("/")
            .last()
            .expect("Invalid arloader manifest key.");

        let number = name.split(".").next().unwrap().to_string();

        let link = value
            .files
            .get(0)
            .expect("Invalid arloader manifest value.")
            .uri
            .clone();

        cache_items.0.insert(
            number,
            CacheItem {
                name: name.to_string(),
                link,
                on_chain: false,
            },
        );
    }

    cache.items = cache_items;

    Ok(())
}

fn read_arloader_manifest(path: &String) -> Result<ArloaderManifest> {
    let file = File::open(path)?;
    let arloader_manifest = serde_json::from_reader(file)?;

    Ok(arloader_manifest)
}

fn generate_config_lines(cache_items: &CacheItems) -> Vec<ConfigLine> {
    let mut config_lines: Vec<ConfigLine> = Vec::new();

    for (_, value) in &cache_items.0 {
        let config_line = value.into_config_line();

        if config_line.is_some() {
            config_lines.push(config_line.unwrap());
        }
    }

    config_lines
}

fn initialize_candy_machine(
    sugar_config: &SugarConfig,
    candy_account: &Keypair,
    candy_machine_data: CandyMachineData,
) -> Result<Signature> {
    let logger = &sugar_config.logger;
    let pid = "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ"
        .parse()
        .expect("Failed to parse PID");

    let client = setup_client(sugar_config)?;

    let program = client.program(pid);
    let payer = program.payer();
    let items_available = candy_machine_data.items_available;

    let candy_account_size = CONFIG_ARRAY_START
        + 4
        + items_available as usize * CONFIG_LINE_SIZE
        + 8
        + 2 * (items_available as usize / 8 + 1);

    info!(
        logger,
        "Initializing candy machine with account size of: {} and address of: {}",
        candy_account_size,
        candy_account.pubkey().to_string()
    );
    let sig = program
        .request()
        .instruction(system_instruction::create_account(
            &payer,
            &candy_account.pubkey(),
            program
                .rpc()
                .get_minimum_balance_for_rent_exemption(candy_account_size)?,
            candy_account_size as u64,
            &program.id(),
        ))
        .signer(candy_account)
        .accounts(nft_accounts::InitializeCandyMachine {
            candy_machine: candy_account.pubkey(),
            wallet: payer,
            authority: payer,
            payer,
            system_program: system_program::id(),
            rent: sysvar::rent::ID,
        })
        .args(nft_instruction::InitializeCandyMachine {
            data: candy_machine_data,
        })
        .send()?;

    // info!(logger, "{}", sig);

    Ok(sig)
}

fn upload_config_lines(
    sugar_config: &SugarConfig,
    config_lines: Vec<ConfigLine>,
    candy_pubkey: Pubkey,
) -> Result<Vec<ConfigStatus>> {
    let index: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));

    let payer = Arc::new(&sugar_config.keypair);
    let logger = &sugar_config.logger;

    let statuses: Arc<Mutex<Vec<ConfigStatus>>> = Arc::new(Mutex::new(Vec::new()));

    let pb = ProgressBar::new(config_lines.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{percent}] {bar:40.cyan/blue}")
            .progress_chars("##-"),
    );

    info!(logger, "Uploading config lines chunks...");
    config_lines
        .par_iter()
        .chunks(CONFIG_CHUNK_SIZE)
        .progress()
        .for_each(|chunk| {
            let index = index.clone();
            let statuses = statuses.clone();

            let mut temp_index: u32 = 0;
            {
                let mut i = index.lock().unwrap();
                temp_index = *i;
                // debug!(logger, "Writing index: {}", temp_index);
                *i += chunk.len() as u32;
            }

            let payer = Arc::clone(&payer);

            let chunk_len = chunk.len();

            match add_config_lines(sugar_config, &candy_pubkey, payer, temp_index, chunk) {
                Ok(_) => {
                    for index in temp_index..(temp_index + chunk_len as u32) {
                        let mut statuses = statuses.lock().unwrap().push(ConfigStatus {
                            index,
                            on_chain: true,
                        });
                    }
                }
                Err(e) => {
                    println!("{}", e);
                    for index in temp_index..(temp_index + chunk_len as u32 as u32) {
                        let mut statuses = statuses.lock().unwrap().push(ConfigStatus {
                            index,
                            on_chain: false,
                        });
                    }
                }
            }
        });

    let statuses = if let Ok(s) = Arc::try_unwrap(statuses) {
        s.into_inner().unwrap()
    } else {
        panic!("Failed to unwrap statuses.");
    };

    Ok(statuses)
}

fn add_config_lines(
    sugar_config: &SugarConfig,
    candy_pubkey: &Pubkey,
    payer: Arc<&Keypair>,
    index: u32,
    // config_slices: &[ConfigLine],
    config_slices: Vec<&ConfigLine>,
) -> Result<()> {
    let pid = "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ"
        .parse()
        .expect("Failed to parse PID");

    let client = setup_client(sugar_config)?;

    let program = client.program(pid);

    // ConfigLine does not implement Clone, so we have to do this.
    let mut config_lines: Vec<ConfigLine> = Vec::new();

    for line in config_slices {
        config_lines.push(ConfigLine {
            name: line.name.clone(),
            uri: line.uri.clone(),
        });
    }

    let sig = program
        .request()
        .accounts(nft_accounts::AddConfigLines {
            candy_machine: *candy_pubkey,
            authority: program.payer(),
        })
        .args(nft_instruction::AddConfigLines {
            index,
            config_lines,
        })
        .signer(*payer)
        .send()?;

    Ok(())
}
