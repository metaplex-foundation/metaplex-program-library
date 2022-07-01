use std::{str::FromStr, thread::sleep, time::Duration};

pub use anchor_client::solana_sdk::hash::Hash;
use anchor_client::{
    solana_sdk::{
        program_pack::{IsInitialized, Pack},
        pubkey::Pubkey,
    },
    Program,
};
pub use anyhow::{anyhow, Result};
use console::{style, Style};
use dialoguer::theme::ColorfulTheme;
pub use indicatif::{ProgressBar, ProgressStyle};
use solana_client::rpc_client::RpcClient;
use spl_token::state::{Account, Mint};

use crate::config::data::Cluster;

/// Hash for devnet cluster
pub const DEVNET_HASH: &str = "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG";

/// Hash for mainnet-beta cluster
pub const MAINNET_HASH: &str = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d";

/// Return the environment of the current connected RPC.
pub fn get_cluster(rpc_client: RpcClient) -> Result<Cluster> {
    let devnet_hash = Hash::from_str(DEVNET_HASH).unwrap();
    let mainnet_hash = Hash::from_str(MAINNET_HASH).unwrap();
    let genesis_hash = rpc_client.get_genesis_hash()?;

    if genesis_hash == devnet_hash {
        Ok(Cluster::Devnet)
    } else if genesis_hash == mainnet_hash {
        Ok(Cluster::Mainnet)
    } else {
        Err(anyhow!(format!(
            "Genesis hash '{}' doesn't match supported Solana clusters for Candy Machine",
            genesis_hash
        )))
    }
}

/// Check that the mint token is a valid address.
pub fn check_spl_token(program: &Program, input: &str) -> Result<Mint> {
    let pubkey = Pubkey::from_str(input)?;
    let token_data = program.rpc().get_account_data(&pubkey)?;
    if token_data.len() != 82 {
        return Err(anyhow!("Invalid spl-token passed in."));
    }
    let token_mint = Mint::unpack_from_slice(&token_data)?;

    if token_mint.is_initialized {
        Ok(token_mint)
    } else {
        Err(anyhow!(format!(
            "The specified spl-token is not initialized: {}",
            input
        )))
    }
}

/// Check that the mint token account is a valid account.
pub fn check_spl_token_account(program: &Program, input: &str) -> Result<()> {
    let pubkey = Pubkey::from_str(input)?;
    let ata_data = program.rpc().get_account_data(&pubkey)?;
    let ata_account = Account::unpack_unchecked(&ata_data)?;

    if IsInitialized::is_initialized(&ata_account) {
        Ok(())
    } else {
        Err(anyhow!(format!(
            "The specified spl-token account is not initialized: {}",
            input
        )))
    }
}

pub fn spinner_with_style() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(120);
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ])
            .template("{spinner:.dim} {msg}"),
    );
    pb
}

pub fn wait_with_spinner_and_countdown(seconds: u64) {
    let pb = spinner_with_style();
    pb.enable_steady_tick(120);
    for i in 0..seconds {
        pb.set_message(
            style(format!("Waiting {} seconds...", seconds - i))
                .dim()
                .to_string(),
        );
        sleep(Duration::from_secs(1));
    }
    pb.finish_and_clear();
}

pub fn progress_bar_with_style(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    // forces the progress bar to show immediately
    pb.tick();
    pb.enable_steady_tick(1000);
    pb.set_style(
        ProgressStyle::default_bar().template("[{elapsed_precise}] {msg}{wide_bar} {pos}/{len}"),
    );
    pb
}

pub fn get_dialoguer_theme() -> ColorfulTheme {
    ColorfulTheme {
        prompt_style: Style::new(),
        checked_item_prefix: style("✔".to_string()).green().force_styling(true),
        unchecked_item_prefix: style("✔".to_string()).black().force_styling(true),
        ..Default::default()
    }
}
