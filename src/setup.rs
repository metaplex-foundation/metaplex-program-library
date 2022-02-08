use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        signature::{keypair::Keypair, read_keypair_file},
    },
    Client, Cluster,
};
use anyhow::Result;

use crate::config::data::SugarConfig;
use crate::parse::*;

pub fn setup_client(sugar_config: &SugarConfig) -> Result<Client> {
    let rpc_url = sugar_config.rpc_url.clone();
    let ws_url = rpc_url.replace("http", "ws");
    let cluster = Cluster::Custom(rpc_url, ws_url);

    let key_bytes = sugar_config.keypair.to_bytes();
    let payer = Keypair::from_bytes(&key_bytes)?;

    let opts = CommitmentConfig::confirmed();
    Ok(Client::new_with_options(cluster, payer, opts))
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
            None => String::from("https://psytrbhymqlkfrhudd.dev.genesysgo.net:8899/"),
        },
    };

    let keypair = match keypair_opt {
        Some(keypair_path) => {
            read_keypair_file(&keypair_path).expect("Failed to read keypair file.")
        }
        None => match sol_config_option {
            Some(ref sol_config) => {
                read_keypair_file(&sol_config.keypair_path).expect("Failed to read keypair file.")
            }
            None => read_keypair_file(&*shellexpand::tilde("~/.config/solana/id.json"))
                .expect("Failed to read keypair file."),
        },
    };

    Ok(SugarConfig { rpc_url, keypair })
}
