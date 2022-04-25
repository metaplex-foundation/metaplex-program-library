use anchor_client::solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    system_instruction, system_program, sysvar,
};
use anchor_lang::prelude::AccountMeta;
use anyhow::Result;
use console::style;
use futures::future::select_all;
use rand::rngs::OsRng;
use spl_associated_token_account::get_associated_token_address;
use std::{cmp, collections::HashSet, str::FromStr, sync::Arc};

use mpl_candy_machine::accounts as nft_accounts;
use mpl_candy_machine::instruction as nft_instruction;
use mpl_candy_machine::{CandyMachineData, ConfigLine, Creator as CandyCreator};
pub use mpl_token_metadata::state::{
    MAX_CREATOR_LIMIT, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH,
};

use crate::cache::*;
use crate::candy_machine::uuid_from_pubkey;
use crate::common::*;
use crate::config::{data::*, parser::get_config_data};
use crate::deploy::data::*;
use crate::deploy::errors::*;
use crate::setup::{setup_client, sugar_setup};
use crate::utils::*;
use crate::validate::parser::{check_name, check_seller_fee_basis_points, check_symbol, check_url};

/// The maximum config line bytes per transaction.
const MAX_TRANSACTION_BYTES: usize = 1000;

/// The maximum number of config lines per transaction.
const MAX_TRANSACTION_LINES: usize = 17;

struct TxInfo {
    client: Arc<Client>,
    candy_pubkey: Pubkey,
    payer: Keypair,
    chunk: Vec<(u32, ConfigLine)>,
}

pub async fn process_deploy(args: DeployArgs) -> Result<()> {
    // loads the cache file (this needs to have been created by
    // the upload command)
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

    // checks that all metadata information are present and have the
    // correct length

    for (index, item) in &cache.items.0 {
        if item.name.is_empty() {
            return Err(DeployError::MissingName(index.to_string()).into());
        } else {
            check_name(&item.name)?;
        }

        if item.metadata_link.is_empty() {
            return Err(DeployError::MissingMetadataLink(index.to_string()).into());
        } else {
            check_url(&item.metadata_link)?;
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

    // checks the candy machine data

    let num_items = config_data.number;
    let hidden = config_data.hidden_settings.is_some();

    if num_items != (cache.items.0.len() as u64) {
        return Err(anyhow!(
            "Number of items ({}) do not match cache items ({})",
            num_items,
            cache.items.0.len()
        ));
    } else {
        check_symbol(&config_data.symbol)?;
        check_seller_fee_basis_points(config_data.seller_fee_basis_points)?;
    }

    let candy_pubkey = if candy_machine_address.is_empty() {
        println!(
            "{} {}Creating candy machine",
            style(if hidden { "[1/1]" } else { "[1/2]" }).bold().dim(),
            CANDY_EMOJI
        );
        info!("Candy machine address is empty, creating new candy machine...");

        let spinner = spinner_with_style();
        spinner.set_message("Creating candy machine...");

        let candy_keypair = Keypair::generate(&mut OsRng);
        let candy_pubkey = candy_keypair.pubkey();

        let uuid = uuid_from_pubkey(&candy_pubkey);
        let candy_data = create_candy_machine_data(&config_data, uuid)?;

        let pid = CANDY_MACHINE_V2.parse().expect("Failed to parse PID");
        let program = client.program(pid);

        let treasury_wallet = match config_data.spl_token {
            Some(spl_token) => {
                let spl_token_account_figured = if config_data.spl_token_account.is_some() {
                    config_data.spl_token_account
                } else {
                    Some(get_associated_token_address(&program.payer(), &spl_token))
                };

                if config_data.sol_treasury_account.is_some() {
                    return Err(anyhow!("If spl-token-account or spl-token is set then sol-treasury-account cannot be set"));
                }

                // validates the mint address of the token accepted as payment
                check_spl_token(&program, &spl_token.to_string())?;

                if let Some(token_account) = spl_token_account_figured {
                    // validates the spl token wallet to receive proceedings from SPL token payments
                    check_spl_token_account(&program, &token_account.to_string())?;
                    token_account
                } else {
                    return Err(anyhow!(
                        "If spl-token is set, spl-token-account must also be set"
                    ));
                }
            }
            None => match config_data.sol_treasury_account {
                Some(sol_treasury_account) => sol_treasury_account,
                None => sugar_config.keypair.pubkey(),
            },
        };

        // all good, let's create the candy machine

        let sig = initialize_candy_machine(
            &config_data,
            &candy_keypair,
            candy_data,
            treasury_wallet,
            program,
        )?;
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
            style(if hidden { "[1/1]" } else { "[1/2]" }).bold().dim(),
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

    if !hidden {
        println!(
            "\n{} {}Writing config lines",
            style("[2/2]").bold().dim(),
            PAPER_EMOJI
        );

        let config_lines = generate_config_lines(num_items, &cache.items)?;

        if config_lines.is_empty() {
            println!("\nAll config lines deployed.");
        } else {
            let errors = upload_config_lines(
                client,
                &sugar_config,
                candy_pubkey,
                &mut cache,
                config_lines,
            )
            .await?;

            if !errors.is_empty() {
                let mut message = String::new();
                message.push_str(&format!(
                    "Failed to deploy all config lines, {0} error(s) occurred:",
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

                return Err(DeployError::AddConfigLineFailed(message).into());
            }
        }
    } else {
        println!("\nCandy machine with hidden settings deployed.");
    }

    Ok(())
}

/// Create the candy machine data struct.
fn create_candy_machine_data(config: &ConfigData, uuid: String) -> Result<CandyMachineData> {
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
    let mut share = 0u32;

    for creator in &config.creators {
        let c = creator.into_candy_format()?;
        share += c.share as u32;

        creators.push(c);
    }

    if creators.is_empty() || creators.len() > (MAX_CREATOR_LIMIT - 1) {
        return Err(anyhow!(
            "The number of creators must be between 1 and {}.",
            MAX_CREATOR_LIMIT - 1,
        ));
    }

    if share != 100 {
        return Err(anyhow!(
            "Creator(s) share must add up to 100, current total {}.",
            share,
        ));
    }

    let data = CandyMachineData {
        uuid,
        price: price_as_lamports(config.price),
        symbol: config.symbol.clone(),
        seller_fee_basis_points: config.seller_fee_basis_points,
        max_supply: 0,
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
fn generate_config_lines(
    num_items: u64,
    cache_items: &CacheItems,
) -> Result<Vec<Vec<(u32, ConfigLine)>>> {
    let mut config_lines: Vec<Vec<(u32, ConfigLine)>> = Vec::new();
    let mut current: Vec<(u32, ConfigLine)> = Vec::new();
    let mut tx_size = 0;

    for i in 0..num_items {
        let item = match cache_items.0.get(&i.to_string()) {
            Some(item) => item,
            None => {
                return Err(
                    DeployError::AddConfigLineFailed(format!("Missing cache item {}", i)).into(),
                );
            }
        };

        if item.on_chain {
            // if the current item is on-chain already, store the previous
            // items as a transaction since we cannot have gaps in the indices
            // to write the config lines
            if !current.is_empty() {
                config_lines.push(current);
                current = Vec::new();
                tx_size = 0;
            }
        } else {
            let config_line = item
                .into_config_line()
                .expect("Could not convert item to config line");

            let size = (2 * STRING_LEN_SIZE) + config_line.name.len() + config_line.uri.len();

            if (tx_size + size) > MAX_TRANSACTION_BYTES || current.len() == MAX_TRANSACTION_LINES {
                // we need a separate tx to not break the size limit
                config_lines.push(current);
                current = Vec::new();
                tx_size = 0;
            }

            tx_size += size;
            current.push((i as u32, config_line));
        }
    }
    // adds the last chunk (if there is one)
    if !current.is_empty() {
        config_lines.push(current);
    }

    Ok(config_lines)
}

/// Send the `initialize_candy_machine` instruction to the candy machine program.
fn initialize_candy_machine(
    config_data: &ConfigData,
    candy_account: &Keypair,
    candy_machine_data: CandyMachineData,
    treasury_wallet: Pubkey,
    program: Program,
) -> Result<Signature> {
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

    let mut tx = program
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
            wallet: treasury_wallet,
            authority: payer,
            payer,
            system_program: system_program::id(),
            rent: sysvar::rent::ID,
        })
        .args(nft_instruction::InitializeCandyMachine {
            data: candy_machine_data,
        });

    if let Some(token) = config_data.spl_token {
        tx = tx.accounts(AccountMeta {
            pubkey: token,
            is_signer: false,
            is_writable: false,
        });
    }

    let sig = tx.send()?;

    Ok(sig)
}

/// Send the config lines to the candy machine program.
async fn upload_config_lines(
    client: Arc<Client>,
    sugar_config: &SugarConfig,
    candy_pubkey: Pubkey,
    cache: &mut Cache,
    config_lines: Vec<Vec<(u32, ConfigLine)>>,
) -> Result<Vec<DeployError>> {
    println!(
        "Sending config line(s) in {} transaction(s): (Ctrl+C to abort)",
        config_lines.len()
    );

    let pb = progress_bar_with_style(config_lines.len() as u64);

    debug!("Num of config line chunks: {:?}", config_lines.len());
    info!("Uploading config lines in chunks...");

    let mut transactions = Vec::new();

    for chunk in config_lines {
        let keypair = bs58::encode(sugar_config.keypair.to_bytes()).into_string();
        let payer = Keypair::from_base58_string(&keypair);

        transactions.push(TxInfo {
            client: client.clone(),
            candy_pubkey,
            payer,
            chunk,
        });
    }

    let mut handles = Vec::new();

    for tx in transactions.drain(0..cmp::min(transactions.len(), PARALLEL_LIMIT)) {
        handles.push(tokio::spawn(async move { add_config_lines(tx).await }));
    }

    let mut errors = Vec::new();

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
                    errors.push(DeployError::AddConfigLineFailed(format!(
                        "Transaction error: {:?}",
                        res.err().unwrap()
                    )));
                }
            }
            (Err(err), _index, remaining) => {
                // user will need to retry the upload
                errors.push(DeployError::AddConfigLineFailed(format!(
                    "Transaction error: {:?}",
                    err
                )));
                // ignoring all errors
                handles = remaining;
            }
        }

        if !transactions.is_empty() {
            // if we are half way through, let spawn more transactions
            if (PARALLEL_LIMIT - handles.len()) > (PARALLEL_LIMIT / 2) {
                for tx in transactions.drain(0..cmp::min(transactions.len(), PARALLEL_LIMIT / 2)) {
                    handles.push(tokio::spawn(async move { add_config_lines(tx).await }));
                }
            }
        }
    }

    if !errors.is_empty() {
        pb.abandon_with_message(format!("{}", style("Deploy failed ").red().bold()));
    } else {
        pb.finish_with_message(format!("{}", style("Deploy successful ").green().bold()));
    }

    Ok(errors)
}

/// Send the `add_config_lines` instruction to the candy machine program.
async fn add_config_lines(tx_info: TxInfo) -> Result<Vec<u32>> {
    let pid = CANDY_MACHINE_V2.parse().expect("Failed to parse PID");
    let program = tx_info.client.program(pid);

    // this will be used to update the cache
    let mut indices: Vec<u32> = Vec::new();
    // configLine does not implement clone, so we have to do this
    let mut config_lines: Vec<ConfigLine> = Vec::new();
    // start index
    let start_index = tx_info.chunk[0].0;

    for (index, line) in tx_info.chunk {
        indices.push(index);
        config_lines.push(line);
    }

    let _sig = program
        .request()
        .accounts(nft_accounts::AddConfigLines {
            candy_machine: tx_info.candy_pubkey,
            authority: program.payer(),
        })
        .args(nft_instruction::AddConfigLines {
            index: start_index,
            config_lines,
        })
        .signer(&tx_info.payer)
        .send()?;

    Ok(indices)
}
