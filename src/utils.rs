use std::{str::FromStr, thread::sleep, time::Duration};

pub use anchor_client::solana_sdk::hash::Hash;
use anchor_client::{
    solana_sdk::{
        account::Account,
        commitment_config::{CommitmentConfig, CommitmentLevel},
        program_pack::{IsInitialized, Pack},
        pubkey::Pubkey,
    },
    Program,
};
pub use anyhow::{anyhow, Result};
use console::{style, Style};
use dialoguer::theme::ColorfulTheme;
pub use indicatif::{ProgressBar, ProgressStyle};
use mpl_token_metadata::ID as TOKEN_METADATA_PROGRAM_ID;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use spl_token::state::{Account as SplAccount, Mint};

use crate::{
    common::*,
    config::{data::Cluster, ConfigData},
};

/// Hash for devnet cluster
pub const DEVNET_HASH: &str = "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG";

/// Hash for mainnet-beta cluster
pub const MAINNET_HASH: &str = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d";

/// Return the environment of the current connected RPC.
pub fn get_cluster(rpc_client: RpcClient) -> Result<Cluster> {
    let devnet_hash = Hash::from_str(DEVNET_HASH).unwrap();
    let mainnet_hash = Hash::from_str(MAINNET_HASH).unwrap();
    let genesis_hash = rpc_client.get_genesis_hash()?;

    Ok(if genesis_hash == devnet_hash {
        Cluster::Devnet
    } else if genesis_hash == mainnet_hash {
        Cluster::Mainnet
    } else {
        Cluster::Unknown
    })
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
            "The specified spl-token is not initialized: {input}",
        )))
    }
}

/// Check that the mint token account is a valid account.
pub fn check_spl_token_account(program: &Program, input: &str) -> Result<()> {
    let pubkey = Pubkey::from_str(input)?;
    let ata_data = program.rpc().get_account_data(&pubkey)?;
    let ata_account = SplAccount::unpack_unchecked(&ata_data)?;

    if IsInitialized::is_initialized(&ata_account) {
        Ok(())
    } else {
        Err(anyhow!(format!(
            "The specified spl-token account is not initialized: {input}",
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

pub fn assert_correct_authority(user_keypair: &Pubkey, update_authority: &Pubkey) -> Result<()> {
    if user_keypair != update_authority {
        return Err(anyhow!(
            "Update authority does not match that of the candy machine."
        ));
    }

    Ok(())
}

pub fn f64_to_u64_safe(f: f64) -> Result<u64, FloatConversionError> {
    if f.fract() != 0.0 {
        return Err(FloatConversionError::Fractional);
    }
    if f <= u64::MIN as f64 || f >= u64::MAX as f64 {
        return Err(FloatConversionError::Overflow);
    }
    Ok(f.trunc() as u64)
}

pub fn get_cm_creator_metadata_accounts(
    client: &RpcClient,
    creator: &str,
    position: usize,
) -> Result<Vec<Pubkey>> {
    let accounts = get_cm_creator_accounts(client, creator, position)?
        .into_iter()
        .map(|(pubkey, _account)| pubkey)
        .collect::<Vec<Pubkey>>();

    Ok(accounts)
}

pub fn get_cm_creator_mint_accounts(
    client: &RpcClient,
    creator: &str,
    position: usize,
) -> Result<Vec<Pubkey>> {
    let accounts = get_cm_creator_accounts(client, creator, position)?
        .into_iter()
        .map(|(_, account)| account.data[33..65].to_vec())
        .map(|data| Pubkey::new(&data))
        .collect::<Vec<Pubkey>>();

    Ok(accounts)
}

fn get_cm_creator_accounts(
    client: &RpcClient,
    creator: &str,
    position: usize,
) -> Result<Vec<(Pubkey, Account)>> {
    if position > 4 {
        error!("CM Creator position cannot be greator than 4");
        std::process::exit(1);
    }

    let config = RpcProgramAccountsConfig {
        filters: Some(vec![RpcFilterType::Memcmp(Memcmp {
            offset: 1 + // key
            32 + // update auth
            32 + // mint
            4 + // name string length
            MAX_NAME_LENGTH + // name
            4 + // uri string length
            MAX_URI_LENGTH + // uri*
            4 + // symbol string length
            MAX_SYMBOL_LENGTH + // symbol
            2 + // seller fee basis points
            1 + // whether or not there is a creators vec
            4 + // creators
            position * // index for each creator
            (
                32 + // address
                1 + // verified
                1 // share
            ),
            bytes: MemcmpEncodedBytes::Base58(creator.to_string()),
            encoding: None,
        })]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: Some(CommitmentConfig {
                commitment: CommitmentLevel::Confirmed,
            }),
            min_context_slot: None,
        },
        with_context: None,
    };

    let results = client.get_program_accounts_with_config(&TOKEN_METADATA_PROGRAM_ID, config)?;

    Ok(results)
}

pub fn get_mint_decimals(program: &Program, config: &ConfigData) -> Result<u8> {
    // If SPL token is used, get the decimals from the token account, otherwise use 9 for SOL.
    if let Some(mint_pubkey) = config.spl_token {
        let mint_account = program.rpc().get_account(&mint_pubkey)?;
        let mint = spl_token::state::Mint::unpack(&mint_account.data)?;
        Ok(mint.decimals)
    } else {
        Ok(9)
    }
}
