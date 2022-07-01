use std::rc::Rc;

use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        signature::{keypair::Keypair, read_keypair_file},
    },
    Client, Cluster,
};
use anyhow::{anyhow, Result};
use tracing::error;

use crate::{
    config::data::SugarConfig,
    constants::{DEFAULT_KEYPATH, DEFAULT_RPC_DEVNET},
    parse::*,
};

pub fn setup_client(sugar_config: &SugarConfig) -> Result<Client> {
    let rpc_url = sugar_config.rpc_url.clone();
    let ws_url = rpc_url.replace("http", "ws");
    let cluster = Cluster::Custom(rpc_url, ws_url);

    let key_bytes = sugar_config.keypair.to_bytes();
    let signer = Rc::new(Keypair::from_bytes(&key_bytes)?);

    let opts = CommitmentConfig::confirmed();
    Ok(Client::new_with_options(cluster, signer, opts))
}

pub fn sugar_setup(
    keypair_opt: Option<String>,
    rpc_url_opt: Option<String>,
) -> Result<SugarConfig> {
    let sol_config_option = parse_solana_config();

    let rpc_url = match rpc_url_opt {
        Some(rpc_url) => rpc_url,
        None => match sol_config_option {
            Some(ref sol_config) => sol_config.json_rpc_url.clone(),
            None => String::from(DEFAULT_RPC_DEVNET),
        },
    };

    let keypair = match keypair_opt {
        Some(keypair_path) => match read_keypair_file(&keypair_path) {
            Ok(keypair) => keypair,
            Err(e) => {
                error!("Failed to read keypair file: {}", e);
                return Err(anyhow!(
                    "Failed to read keypair file: {}, {}",
                    keypair_path,
                    e
                ));
            }
        },

        None => match sol_config_option {
            Some(ref sol_config) => match read_keypair_file(&sol_config.keypair_path) {
                Ok(keypair) => keypair,
                Err(e) => {
                    error!(
                        "Failed to read keypair file: {}, {}",
                        &sol_config.keypair_path, e
                    );
                    return Err(anyhow!(
                        "Failed to read keypair file: {}, {}",
                        &sol_config.keypair_path,
                        e
                    ));
                }
            },
            None => match read_keypair_file(&*shellexpand::tilde(DEFAULT_KEYPATH)) {
                Ok(keypair) => keypair,
                Err(e) => {
                    error!("Failed to read keypair file: {}, {}", DEFAULT_KEYPATH, e);
                    return Err(anyhow!(
                        "Failed to read keypair file: {}, {}",
                        DEFAULT_KEYPATH,
                        e
                    ));
                }
            },
        },
    };

    Ok(SugarConfig { rpc_url, keypair })
}
