use anchor_client::solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    system_instruction, system_program, sysvar,
};
use anyhow::Result;
use console::style;
use indicatif::{ParallelProgressIterator, ProgressBar};
use rand::rngs::OsRng;
use rayon::prelude::*;
use std::{
    fs::File,
    path::Path,
    str::FromStr,
    sync::{Arc, Mutex},
};

use mpl_candy_machine::accounts as nft_accounts;
use mpl_candy_machine::instruction as nft_instruction;
use mpl_candy_machine::{CandyMachineData, ConfigLine, Creator as CandyCreator};

use crate::cache::*;
use crate::candy_machine::{uuid_from_pubkey, ConfigStatus};
use crate::common::*;
use crate::config::{data::*, parser::get_config_data};
use crate::setup::{setup_client, sugar_setup};
use crate::upload::data::*;
use crate::validate::format::Metadata;

pub fn process_upload(args: UploadArgs) -> Result<()> {
    let sugar_config = match sugar_setup(args.keypair, args.rpc_url) {
        Ok(sugar_config) => sugar_config,
        Err(err) => {
            return Err(SetupError::SugarSetupError(err.to_string()).into());
        }
    };

    let client = Arc::new(setup_client(&sugar_config)?);

    let mut cache = load_cache(&args.cache)?;
    let config_data = get_config_data(&args.config)?;
    let candy_machine_address = &cache.program.candy_machine;

    let candy_pubkey = if candy_machine_address.is_empty() {
        println!(
            "{} {}Creating candy machine",
            style("[1/2]").bold().dim(),
            CANDY_EMOJI
        );
        info!("Candy machine address is empty, creating new candy machine...");

        let candy_keypair = Keypair::generate(&mut OsRng);
        let candy_pubkey = candy_keypair.pubkey();

        let uuid = uuid_from_pubkey(&candy_pubkey);
        let metadata = get_metadata_from_first_json(&args.assets_dir)?;
        let candy_data = create_candy_machine_data(&config_data, uuid, metadata)?;

        let sig = initialize_candy_machine(&candy_keypair, candy_data, client.clone())?;
        info!("Candy machine initialized with sig: {}", sig);
        info!(
            "Candy machine created with address: {}",
            &candy_pubkey.to_string()
        );

        cache.program = CacheProgram::new_from_cm(&candy_pubkey);
        cache.write_to_file(&args.cache)?;
        candy_pubkey
    } else {
        println!(
            "{} {}Loading candy machine",
            style("[1/2]").bold().dim(),
            CANDY_EMOJI
        );

        match Pubkey::from_str(candy_machine_address) {
            Ok(pubkey) => pubkey,
            Err(_err) => {
                error!(
                    "Invalid candy machine address in cache file: {}!",
                    candy_machine_address
                );
                return Err(CacheError::InvalidCandyMachineAddress(
                    candy_machine_address.to_string(),
                )
                .into());
            }
        }
    };

    println!("Candy machine ID: {}", candy_pubkey);
    info!("Uploading config lines...");

    println!(
        "\n{} {}Uploading config lines",
        style("[2/2]").bold().dim(),
        PAPER_EMOJI
    );

    let num_items = config_data.number;
    let config_lines = generate_config_lines(num_items, &cache.items);
    let config_statuses = upload_config_lines(&sugar_config, config_lines, candy_pubkey, client)?;

    for status in config_statuses {
        let index: String = status.index.to_string();
        let mut item = cache.items.0.get_mut(&index).unwrap();
        item.on_chain = status.on_chain;
    }

    cache.write_to_file(&args.cache)?;

    println!("\n{}", style("[Completed]").bold().dim());

    Ok(())
}

fn get_metadata_from_first_json(assets_dir: &str) -> Result<Metadata> {
    let f = File::open(Path::new(assets_dir).join("0.json"))?;
    let metadata: Metadata = match serde_json::from_reader(f) {
        Ok(metadata) => metadata,
        Err(err) => {
            let error = anyhow!("Error parsing metadata from 0.json: {}", err);
            error!("{:?}", error);
            return Err(error);
        }
    };

    Ok(metadata)
}

fn create_candy_machine_data(
    config: &ConfigData,
    uuid: String,
    metadata: Metadata,
) -> Result<CandyMachineData> {
    let go_live_date = Some(go_live_date_as_timestamp(&config.go_live_date)?);

    let end_settings = config.end_settings.as_ref().map(|s| s.into_candy_format());

    let whitelist_mint_settings = config
        .whitelist_mint_settings
        .as_ref()
        .map(|s| s.into_candy_format());

    let hidden_settings = config
        .hidden_settings
        .as_ref()
        .map(|s| s.into_candy_format());

    let gatekeeper = config
        .gatekeeper
        .as_ref()
        .map(|gatekeeper| gatekeeper.into_candy_format());

    let mut creators: Vec<CandyCreator> = Vec::new();

    for creator in metadata.properties.creators {
        creators.push(creator.into_candy_format()?);
    }

    let data = CandyMachineData {
        uuid,
        price: price_as_lamports(config.price),
        symbol: metadata.symbol,
        seller_fee_basis_points: metadata.seller_fee_basis_points,
        max_supply: config.number,
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

#[allow(dead_code)]
fn populate_cache_with_links(
    cache: &mut Cache,
    arloader_manifest: &ArloaderManifest,
) -> Result<()> {
    let mut cache_items: CacheItems = CacheItems(IndexMap::new());

    for (key, value) in &arloader_manifest.0 {
        let name = key
            .split('/')
            .last()
            .expect("Invalid arloader manifest key.");

        let number = name.split('.').next().unwrap().to_string();

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

fn _read_arloader_manifest(path: &str) -> Result<ArloaderManifest> {
    let file = File::open(path)?;
    let arloader_manifest = serde_json::from_reader(file)?;

    Ok(arloader_manifest)
}

fn generate_config_lines(num_items: u64, cache_items: &CacheItems) -> Vec<Vec<(u32, ConfigLine)>> {
    let mut config_lines: Vec<Vec<(u32, ConfigLine)>> = Vec::new();

    // Populate with empty chunks
    for _ in 0..num_items {
        config_lines.push(Vec::new());
    }

    for (key, value) in &cache_items.0 {
        let config_line = value.into_config_line();

        let key = key.parse::<usize>().unwrap();

        let chunk_index = key / CONFIG_CHUNK_SIZE;

        if let Some(config_line) = config_line {
            let chunk = config_lines
            .get_mut(chunk_index)
            .expect("Invalid config line index! Check that your config item number matches the number of assets you're trying to upload.");
            chunk.push((key as u32, config_line));
        }
    }

    config_lines
}

fn initialize_candy_machine(
    candy_account: &Keypair,
    candy_machine_data: CandyMachineData,
    client: Arc<Client>,
) -> Result<Signature> {
    let pid = CANDY_MACHINE_V2.parse().expect("Failed to parse PID");

    let program = client.program(pid);
    let payer = program.payer();
    let items_available = candy_machine_data.items_available;

    let candy_account_size = CONFIG_ARRAY_START
        + 4
        + items_available as usize * CONFIG_LINE_SIZE
        + 8
        + 2 * (items_available as usize / 8 + 1);

    info!(
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

    Ok(sig)
}

fn upload_config_lines(
    sugar_config: &SugarConfig,
    config_lines: Vec<Vec<(u32, ConfigLine)>>,
    candy_pubkey: Pubkey,
    client: Arc<Client>,
) -> Result<Vec<ConfigStatus>> {
    let payer = Arc::new(&sugar_config.keypair);
    let statuses: Arc<Mutex<Vec<ConfigStatus>>> = Arc::new(Mutex::new(Vec::new()));

    println!(
        "Sending {} config line(s): (Ctrl+C to abort)",
        config_lines.len()
    );
    let pb = ProgressBar::new(config_lines.len() as u64);

    debug!("Num of config lines: {:?}", config_lines.len());
    info!("Uploading config lines in chunks...");

    config_lines
        .into_iter()
        // Skip empty chunks
        .filter(|chunk| !chunk.is_empty())
        .collect::<Vec<Vec<(u32, ConfigLine)>>>()
        .par_iter()
        .progress()
        .for_each(|chunk| {
            let statuses = statuses.clone();
            let payer = Arc::clone(&payer);

            match add_config_lines(client.clone(), &candy_pubkey, &payer, chunk) {
                Ok(_) => {
                    for (index, _) in chunk {
                        let _statuses = statuses.lock().unwrap().push(ConfigStatus {
                            index: *index as u32,
                            on_chain: true,
                        });
                    }
                }
                Err(e) => {
                    info!("{}", e);
                    for (index, _) in chunk {
                        let _statuses = statuses.lock().unwrap().push(ConfigStatus {
                            index: *index as u32,
                            on_chain: false,
                        });
                    }
                }
            }

            pb.inc(1);
        });

    pb.finish();

    let statuses = if let Ok(s) = Arc::try_unwrap(statuses) {
        s.into_inner().unwrap()
    } else {
        panic!("Failed to unwrap statuses.");
    };

    Ok(statuses)
}

fn add_config_lines(
    client: Arc<Client>,
    candy_pubkey: &Pubkey,
    payer: &Keypair,
    chunk: &[(u32, ConfigLine)],
) -> Result<()> {
    let pid = CANDY_MACHINE_V2.parse().expect("Failed to parse PID");

    let program = client.program(pid);

    // ConfigLine does not implement Clone, so we have to do this.
    let mut config_lines: Vec<ConfigLine> = Vec::new();

    // First index
    let index = chunk[0].0;

    for (_, line) in chunk {
        config_lines.push(ConfigLine {
            name: line.name.clone(),
            uri: line.uri.clone(),
        });
    }

    let _sig = program
        .request()
        .accounts(nft_accounts::AddConfigLines {
            candy_machine: *candy_pubkey,
            authority: program.payer(),
        })
        .args(nft_instruction::AddConfigLines {
            index,
            config_lines,
        })
        .signer(payer)
        .send()?;

    Ok(())
}
