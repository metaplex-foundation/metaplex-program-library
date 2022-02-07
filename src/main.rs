#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use clap::Parser;
use slog::{Drain, Logger};
use std::{fs::File, str::FromStr};

use mpl_candy_machine::CandyMachineData;

use sugar::cache::Cache;
use sugar::candy_machine::{get_candy_machine_state, print_candy_machine_state};
use sugar::cli::{Cli, Commands};
use sugar::mint::{process_mint_one, MintOneArgs};
use sugar::setup::sugar_setup;
use sugar::upload::{process_upload, UploadArgs};
use sugar::upload_assets::{process_upload_assets, UploadAssetsArgs};
use sugar::validate::{process_validate, ValidateArgs};
use sugar::verify::{process_verify, VerifyArgs};
use sugar::withdraw::{process_withdraw,process_withdraw_all, WithdrawArgs,WithdrawAllArgs};

pub fn default_candy_data() -> CandyMachineData {
    CandyMachineData {
        uuid: String::default(),
        price: u64::default(),
        symbol: String::default(),
        seller_fee_basis_points: u16::default(),
        max_supply: u64::default(),
        is_mutable: false,
        retain_authority: false,
        go_live_date: None,
        end_settings: None,
        creators: vec![],
        hidden_settings: None,
        whitelist_mint_settings: None,
        items_available: 1000,
        gatekeeper: None,
    }
}

fn setup_logging() -> Logger {
    // let log_path = "sugar.log";
    // let file = OpenOptions::new()
    //     .create(true)
    //     .write(true)
    //     .truncate(true)
    //     .open(log_path)
    //     .expect("Failed to create log file");

    // let decorator = slog_term::PlainDecorator::new(file);
    let decorator = slog_term::TermDecorator::new().build();

    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    slog::Logger::root(drain, o!())
}

#[tokio::main(worker_threads = 4)]
async fn main() -> Result<()> {
    let logger = setup_logging();
    info!(logger, "Lend me some sugar, I am your neighbor.");

    let cli = Cli::parse();

    match cli.command {
        Commands::MintOne { keypair, rpc_url } => process_mint_one(MintOneArgs {
            logger,
            keypair,
            rpc_url,
        })?,
        Commands::Upload {
            assets_dir,
            arloader_manifest,
            config,
            keypair,
            rpc_url,
            cache,
        } => process_upload(UploadArgs {
            logger,
            assets_dir,
            arloader_manifest,
            config,
            keypair,
            rpc_url,
            cache,
        })?,
        Commands::UploadAssets {
            assets_dir,
            config,
            keypair,
            rpc_url,
            cache,
        } => {
            process_upload_assets(UploadAssetsArgs {
                logger,
                assets_dir,
                config,
                keypair,
                rpc_url,
                cache,
            })
            .await?
        }
        Commands::Test => process_test_command(logger),
        Commands::Validate { assets_dir, strict } => process_validate(ValidateArgs {
            logger,
            assets_dir,
            strict,
        })?,
        Commands::Withdraw {
            candy_machine,
            keypair,
            rpc_url,
        } => process_withdraw(WithdrawArgs {
            logger,
            candy_machine,
            keypair,
            rpc_url,
        })?,
        Commands::WithdrawAll {
            keypair,
            rpc_url,
        } => process_withdraw_all(WithdrawAllArgs {
            logger,
            keypair,
            rpc_url,
        })?,
        Commands::Verify {
            keypair,
            rpc_url,
            cache,
        } => process_verify(VerifyArgs {
            logger,
            keypair,
            rpc_url,
            cache,
        })?,
    }

    Ok(())
}

fn process_test_command(logger: Logger) {
    let sugar_config = sugar_setup(logger, None, None).unwrap();
    let file = File::open("cache.json").unwrap();
    let cache: Cache = serde_json::from_reader(file).unwrap();

    let candy_machine_id = Pubkey::from_str(&cache.program.candy_machine).unwrap();
    let state = get_candy_machine_state(&sugar_config, &candy_machine_id).unwrap();

    print_candy_machine_state(state);
}
