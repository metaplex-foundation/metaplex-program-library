use anchor_client::solana_sdk::{
    program_pack::{Pack, IsInitialized},
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    system_instruction, system_program, sysvar,
};
use anyhow::Result;
use console::style;
use futures::future::select_all;
use rand::rngs::OsRng;
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::{Account, Mint};
use std::{str::FromStr, sync::Arc};

use mpl_candy_machine::accounts as nft_accounts;
use mpl_candy_machine::instruction as nft_instruction;
use mpl_candy_machine::{CandyMachineData, ConfigLine, Creator as CandyCreator};

use crate::cache::*;
use crate::candy_machine::uuid_from_pubkey;
use crate::common::*;
use crate::config::{data::*, parser::get_config_data};
use crate::deploy::data::*;
use crate::deploy::errors::*;
use crate::setup::{setup_client, sugar_setup};
use crate::utils::*;
use crate::validate::format::Metadata;

/// Name of the first metadata file.
const METADATA_FILE: &str = "0.json";

pub async fn process_deploy(args: DeployArgs) -> Result<()> {
    // loads the cache file (this needs to have been created by
    // the upload_assets command)
    let mut cache = load_cache(&args.cache, false)?;

    if cache.items.0.is_empty() {
        println!(
            "{}",
            style("No cache items found - run 'upload' to create the cache file first.")
                .red()
                .bold()
        );

        // nothing else to do, just tell that the cache file was not found (or empty)
        return Err(CacheError::CacheFileNotFound(args.cache).into());
    }

    // checks that all metadata links are present
    for (index, item) in &cache.items.0 {
        if item.metadata_link.is_empty() {
            return Err(UploadError::MissingMetadataLink(index.to_string()).into());
        }
    }

    let sugar_config = match sugar_setup(args.keypair, args.rpc_url) {
        Ok(sugar_config) => sugar_config,
        Err(err) => {
            return Err(SetupError::SugarSetupError(err.to_string()).into());
        }
    };
    let client = Arc::new(setup_client(&sugar_config)?);
    let config_data = get_config_data(&args.config)?;

    let candy_machine_address = &cache.program.candy_machine;

    let num_items = config_data.number;

    // Do this check before creating a candy machine.
    if num_items != (cache.items.0.len() as u64) {
        return Err(anyhow!(
            "Number of items ({}) do not match cache items ({})",
            num_items,
            cache.items.0.len()
        ));
    }

    let candy_pubkey = if candy_machine_address.is_empty() {
        println!(
            "{} {}Creating candy machine",
            style("[1/2]").bold().dim(),
            CANDY_EMOJI
        );
        info!("Candy machine address is empty, creating new candy machine...");

        let spinner = spinner_with_style();
        spinner.set_message("Creating candy machine...");

        let candy_keypair = Keypair::generate(&mut OsRng);
        let candy_pubkey = candy_keypair.pubkey();

        // loads the metadata of the first cache item
        let metadata: Metadata = {
            let f = File::open(Path::new(&args.assets_dir).join(METADATA_FILE))?;
            match serde_json::from_reader(f) {
                Ok(metadata) => metadata,
                Err(err) => {
                    let error = anyhow!("Error parsing metadata ({}): {}", METADATA_FILE, err);
                    error!("{:?}", error);
                    return Err(error);
                }
            }
        };

        let uuid = uuid_from_pubkey(&candy_pubkey);
        let candy_data = create_candy_machine_data(&config_data, uuid, metadata)?;

        let pid = CANDY_MACHINE_V2.parse().expect("Failed to parse PID");

        let program = client.program(pid);

        let payer = program.payer();

        if config_data.spl_token.is_some() {
            let spl_token = config_data.spl_token.unwrap();
            let spl_token_account_figured = if config_data.spl_token_account.is_some() {
                config_data.spl_token_account
            } else {
                Some(get_associated_token_address(&payer, &spl_token))
            };

            let token_data = program.rpc().get_account_data(&spl_token)?;

            let token_mint = Mint::unpack_from_slice(&token_data)?;
            if !token_mint.is_initialized {
                let error = anyhow!("The specified spl-token is not initialized.");
                error!("{:?}", error);
                return Err(error);
            }

            let ata_data = program
                .rpc()
                .get_account_data(&spl_token_account_figured.unwrap())?;
            let ata_account = Account::unpack_unchecked(&ata_data)?;
            let is_initialized = IsInitialized::is_initialized(&ata_account);
            if !is_initialized {
                let error = anyhow!("The specified spl-token is not initialized.");
                error!("{:?}", error);
                return Err(error);
            }

            if config_data.sol_treasury_account.is_some() {
                let error = anyhow!("If spl-token-account or spl-token is set then sol-treasury-account cannot be set");
                error!("{:?}", error);
                return Err(error);
            }

            if spl_token_account_figured.is_none() {
                let error = anyhow!("If spl-token is set, spl-token-account must also be set");
                error!("{:?}", error);
                return Err(error);
            }
        }

        let sig = initialize_candy_machine(&candy_keypair, candy_data, client.clone())?;
        info!("Candy machine initialized with sig: {}", sig);
        info!(
            "Candy machine created with address: {}",
            &candy_pubkey.to_string()
        );

        cache.program = CacheProgram::new_from_cm(&candy_pubkey);
        cache.sync_file()?;

        spinner.finish_and_clear();

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

    println!("{} {}", style("Candy machine ID:").bold(), candy_pubkey);

    println!(
        "\n{} {}Writing config lines",
        style("[2/2]").bold().dim(),
        PAPER_EMOJI
    );

    let config_lines = generate_config_lines(num_items, &cache.items);

    let completed = if config_lines.is_empty() {
        println!("\n{}All config lines deployed", COMPLETE_EMOJI);
        true
    } else {
        upload_config_lines(
            client,
            &sugar_config,
            candy_pubkey,
            &mut cache,
            config_lines,
        )
        .await?
    };

    if completed {
        println!("\n{}", style("[Completed]").green().bold());
    } else {
        println!("\n{}", style("[Re-run needed]").red().bold())
    }

    Ok(())
}

/// Create the candy machine data struct.
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

    for creator in &config.creators {
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

/// Determine the config lines that need to be uploaded.
fn generate_config_lines(num_items: u64, cache_items: &CacheItems) -> Vec<Vec<(u32, ConfigLine)>> {
    let mut config_lines: Vec<Vec<(u32, ConfigLine)>> = Vec::new();
    let mut on_chain = HashMap::<usize, u32>::new();

    // initializes each chunck
    for _ in 0..num_items {
        config_lines.push(Vec::new());
    }

    for (key, value) in &cache_items.0 {
        let config_line = value.into_config_line();
        let key = key.parse::<usize>().unwrap();
        let chunk_index = key / CONFIG_CHUNK_SIZE;

        // checks if the config line is already on chain
        if value.on_chain {
            on_chain.insert(
                chunk_index,
                match on_chain.get(&chunk_index) {
                    Some(value) => value + 1,
                    None => 1,
                },
            );
        }

        if let Some(config_line) = config_line {
            let chunk = config_lines.get_mut(chunk_index).unwrap();
            chunk.push((key as u32, config_line));
        }
    }

    // removes the chunks where all config lines are on chain already
    for (index, value) in on_chain {
        if value == (config_lines.get(index).unwrap().len() as u32) {
            config_lines.remove(index);
        }
    }

    // removes any empty chunk
    config_lines.retain(|chunk| !chunk.is_empty());

    config_lines
}

/// Send the `initialize_candy_machine` instruction to the candy machine program.
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

/// Send the config lines to the candy machine program.
async fn upload_config_lines(
    client: Arc<Client>,
    sugar_config: &SugarConfig,
    candy_pubkey: Pubkey,
    cache: &mut Cache,
    config_lines: Vec<Vec<(u32, ConfigLine)>>,
) -> Result<bool> {
    println!(
        "Sending config line(s) in {} transaction(s): (Ctrl+C to abort)",
        config_lines.len()
    );

    let pb = progress_bar_with_style(config_lines.len() as u64);

    debug!("Num of config line chunks: {:?}", config_lines.len());
    info!("Uploading config lines in chunks...");

    let mut handles = Vec::new();

    for chunk in config_lines {
        let keypair = bs58::encode(sugar_config.keypair.to_bytes()).into_string();
        let c = client.clone();

        let handle = tokio::spawn(async move {
            let payer = Keypair::from_base58_string(&keypair);
            add_config_lines(c, &candy_pubkey, &payer, &chunk).await
        });

        handles.push(handle);
    }

    let mut failed = false;

    while !handles.is_empty() {
        match select_all(handles).await {
            (Ok(res), _index, remaining) => {
                // independently if the upload was successful or not
                // we continue to try the remaining ones
                handles = remaining;

                if res.is_ok() {
                    let indices = res?;

                    for index in indices {
                        let item = cache.items.0.get_mut(&index.to_string()).unwrap();
                        item.on_chain = true;
                    }
                    // saves the progress to the cache file
                    cache.sync_file()?;
                    // updates the progress bar
                    pb.inc(1);
                } else {
                    // user will need to retry the upload
                    debug!("add_config_lines error: {:?}", res);
                    failed = true;
                }
            }
            (Err(err), _index, remaining) => {
                failed = true;
                debug!("add_config_lines error: {}", err);
                // ignoring all errors
                handles = remaining;
            }
        }
    }

    if failed {
        pb.abandon_with_message(format!(
            "{}",
            style("Error: re-run the deploy to complete the process ")
                .red()
                .bold()
        ));
    } else {
        pb.finish_with_message(format!("{}", style("Deploy successful ").green().bold()));
    }

    Ok(!failed)
}

/// Send the `add_config_lines` instruction to the candy machine program.
async fn add_config_lines(
    client: Arc<Client>,
    candy_pubkey: &Pubkey,
    payer: &Keypair,
    chunk: &[(u32, ConfigLine)],
) -> Result<Vec<u32>> {
    let pid = CANDY_MACHINE_V2.parse().expect("Failed to parse PID");
    let program = client.program(pid);

    // this will be used to update the cache
    let mut indices: Vec<u32> = Vec::new();
    // configLine does not implement clone, so we have to do this
    let mut config_lines: Vec<ConfigLine> = Vec::new();

    for (index, line) in chunk {
        indices.push(*index);
        config_lines.push(ConfigLine {
            name: line.name.clone(),
            uri: line.uri.clone(),
        });
    }

    // first index
    let index = chunk[0].0;

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
        .send();

    Ok(indices)
}
