use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::{anyhow, Result};
use clap::Parser;
use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
    str::FromStr,
};
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{self, filter::LevelFilter, prelude::*, EnvFilter};

use sugar_cli::cache::Cache;
use sugar_cli::candy_machine::{get_candy_machine_state, print_candy_machine_state};
use sugar_cli::cli::{Cli, Commands};
use sugar_cli::constants::DEFAULT_CACHE;
use sugar_cli::mint::{process_mint, MintArgs};
use sugar_cli::setup::sugar_setup;
use sugar_cli::update::{process_update, UpdateArgs};
use sugar_cli::upload::{process_upload, UploadArgs};
use sugar_cli::upload_assets::{process_upload_assets, UploadAssetsArgs};
use sugar_cli::validate::{process_validate, ValidateArgs};
use sugar_cli::verify::{process_verify, VerifyArgs};
use sugar_cli::withdraw::{process_withdraw, WithdrawArgs};

fn setup_logging(level: Option<EnvFilter>) -> Result<()> {
    // Log path; change this to be dynamic for multiple OSes.
    // Log in current directory for now.
    let log_path = PathBuf::from("sugar.log");

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&log_path)
        .unwrap();

    // Prioritize user-provided level, otherwise read from RUST_LOG env var for log level, fall back to "info" if not set.
    let env_filter = if let Some(filter) = level {
        filter
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("off"))
    };

    let formatting_layer = BunyanFormattingLayer::new("sugar".into(), file);
    // let debug_log = tracing_subscriber::fmt::layer().with_writer(Arc::new(file));
    let stdout_layer = tracing_subscriber::fmt::layer().pretty();

    let level_filter = LevelFilter::from_str(&env_filter.to_string())?;

    let subscriber = tracing_subscriber::registry()
        .with(stdout_layer.with_filter(level_filter))
        .with(formatting_layer)
        .with(JsonStorageLayer);

    set_global_default(subscriber).expect("Failed to set global default subscriber");

    Ok(())
}

#[tokio::main(worker_threads = 4)]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let log_level_error: Result<()> = Err(anyhow!(
        "Invalid log level: {:?}.\n Valid levels are: trace, debug, info, warn, error.",
        cli.log_level
    ));

    if let Some(user_filter) = cli.log_level {
        let filter = match EnvFilter::from_str(&user_filter) {
            Ok(filter) => filter,
            Err(_) => return log_level_error,
        };
        setup_logging(Some(filter))?;
    } else {
        setup_logging(None)?;
    }

    tracing::info!("Lend me some sugar, I am your neighbor.");

    match cli.command {
        Commands::Mint {
            keypair,
            rpc_url,
            cache,
            number,
        } => process_mint(MintArgs {
            keypair,
            rpc_url,
            cache,
            number,
        })?,
        Commands::Update {
            config,
            keypair,
            rpc_url,
            cache,
            new_authority,
        } => process_update(UpdateArgs {
            config,
            keypair,
            rpc_url,
            cache,
            new_authority,
        })?,
        Commands::Upload {
            assets_dir,
            arloader_manifest,
            config,
            keypair,
            rpc_url,
            cache,
        } => process_upload(UploadArgs {
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
                assets_dir,
                config,
                keypair,
                rpc_url,
                cache,
            })
            .await?
        }
        Commands::Test => process_test_command(),
        Commands::Validate { assets_dir, strict } => {
            process_validate(ValidateArgs { assets_dir, strict })?
        }
        Commands::Withdraw {
            candy_machine,
            keypair,
            rpc_url,
            list,
        } => process_withdraw(WithdrawArgs {
            candy_machine,
            keypair,
            rpc_url,
            list,
        })?,
        Commands::Verify {
            keypair,
            rpc_url,
            cache,
        } => process_verify(VerifyArgs {
            keypair,
            rpc_url,
            cache,
        })?,
    }

    Ok(())
}

fn process_test_command() {
    let sugar_config = sugar_setup(None, None).unwrap();
    let file = File::open(DEFAULT_CACHE).unwrap();
    let cache: Cache = serde_json::from_reader(file).unwrap();

    let candy_machine_id = Pubkey::from_str(&cache.program.candy_machine).unwrap();
    let state = get_candy_machine_state(&sugar_config, &candy_machine_id).unwrap();

    print_candy_machine_state(state);
}
