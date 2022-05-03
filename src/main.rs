use anyhow::{anyhow, Result};
use clap::Parser;
use console::style;
use std::{
    fs::OpenOptions,
    path::PathBuf,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{self, filter::LevelFilter, prelude::*, EnvFilter};

use sugar_cli::cli::{Cli, Commands};
use sugar_cli::constants::{COMPLETE_EMOJI, ERROR_EMOJI};
use sugar_cli::create_config::{process_create_config, CreateConfigArgs};
use sugar_cli::deploy::{process_deploy, DeployArgs};
use sugar_cli::launch::{process_launch, LaunchArgs};
use sugar_cli::mint::{process_mint, MintArgs};
use sugar_cli::show::{process_show, ShowArgs};
use sugar_cli::update::{process_update, UpdateArgs};
use sugar_cli::upload::{process_upload, UploadArgs};
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

    // Prioritize user-provided level, otherwise read from RUST_LOG env var for log level, fall back to "tracing" if not set.
    let env_filter = if let Some(filter) = level {
        filter
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("trace"))
    };

    let formatting_layer = BunyanFormattingLayer::new("sugar".into(), file);
    let level_filter = LevelFilter::from_str(&env_filter.to_string())?;

    let subscriber = tracing_subscriber::registry()
        .with(formatting_layer.with_filter(level_filter))
        .with(JsonStorageLayer);

    set_global_default(subscriber).expect("Failed to set global default subscriber");

    Ok(())
}

#[tokio::main(worker_threads = 4)]
async fn main() {
    match run().await {
        Ok(()) => {
            println!(
                "\n{}{}",
                COMPLETE_EMOJI,
                style("Command successful.").green().bold().dim()
            );
        }
        Err(err) => {
            println!(
                "\n{}{} {}",
                ERROR_EMOJI,
                style("Error running command (re-run needed):").red(),
                err,
            );
            // finished the program with an error code to the OS
            std::process::exit(1);
        }
    }
}

async fn run() -> Result<()> {
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

    let interrupted = Arc::new(AtomicBool::new(true));
    let ctrl_handler = interrupted.clone();

    ctrlc::set_handler(move || {
        if ctrl_handler.load(Ordering::SeqCst) {
            // we really need to exit
            println!(
                "\n\n{}{} Operation aborted.",
                ERROR_EMOJI,
                style("Error running command (re-run needed):").red(),
            );
            // finished the program with an error code to the OS
            std::process::exit(1);
        }
        // signal that we want to exit
        ctrl_handler.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    match cli.command {
        Commands::CreateConfig {
            config,
            keypair,
            rpc_url,
            assets_dir,
        } => process_create_config(CreateConfigArgs {
            config,
            keypair,
            rpc_url,
            assets_dir,
        })?,
        Commands::Launch {
            assets_dir,
            config,
            keypair,
            rpc_url,
            cache,
            strict,
        } => {
            process_launch(LaunchArgs {
                assets_dir,
                config,
                keypair,
                rpc_url,
                cache,
                strict,
                interrupted: interrupted.clone(),
            })
            .await?
        }
        Commands::Mint {
            keypair,
            rpc_url,
            cache,
            number,
            candy_machine,
        } => process_mint(MintArgs {
            keypair,
            rpc_url,
            cache,
            number,
            candy_machine,
        })?,
        Commands::Update {
            config,
            keypair,
            rpc_url,
            cache,
            new_authority,
            candy_machine,
        } => process_update(UpdateArgs {
            config,
            keypair,
            rpc_url,
            cache,
            new_authority,
            candy_machine,
        })?,
        Commands::Deploy {
            config,
            keypair,
            rpc_url,
            cache,
        } => {
            process_deploy(DeployArgs {
                config,
                keypair,
                rpc_url,
                cache,
                interrupted: interrupted.clone(),
            })
            .await?
        }
        Commands::Upload {
            assets_dir,
            config,
            keypair,
            rpc_url,
            cache,
        } => {
            process_upload(UploadArgs {
                assets_dir,
                config,
                keypair,
                rpc_url,
                cache,
                interrupted: interrupted.clone(),
            })
            .await?
        }
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
        Commands::Show {
            keypair,
            rpc_url,
            cache,
            candy_machine,
        } => process_show(ShowArgs {
            keypair,
            rpc_url,
            cache,
            candy_machine,
        })?,
    }

    Ok(())
}
