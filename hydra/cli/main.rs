use crate::cli_api::init_api;
use anchor_client::anchor_lang::solana_program::example_mocks::solana_sdk::signature::Keypair;
use anchor_client::anchor_lang::AccountDeserialize;
use anchor_client::solana_client::client_error::ClientError;
use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};

use anchor_client::solana_client::rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType};

use clap::{ArgMatches, Error, ErrorKind};
use hydra::state::{Fanout, FanoutMint};
use solana_account_decoder::UiAccountEncoding;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::read_keypair_file;

use std::str::FromStr;
use std::time::Duration;

mod cli_api;

fn setup_connection(app: &ArgMatches) -> (RpcClient, Keypair) {
    let json = app
        .value_of("rpc")
        .unwrap_or(&"wss://api.devnet.solana.com".to_owned())
        .to_owned();
    let _wss = json.replace("https", "wss");

    let _payer = read_keypair_file(app.value_of("keypair").unwrap()).unwrap();
    let timeout = Duration::from_secs(30);

    (
        RpcClient::new_with_timeout_and_commitment(json, timeout, CommitmentConfig::confirmed()),
        Keypair,
    )
}

#[derive(Debug)]
struct HydraMint {
    fanout_mint: FanoutMint,
    address: Pubkey,
}

#[derive(Debug)]
struct HydraObject {
    pub fanout: Fanout,
    pub address: Pubkey,
    pub children: Option<HydraMint>,
}

fn main() {
    let app = init_api().get_matches();
    let (rpc, _payer) = setup_connection(&app);
    let (rpc2, _payer) = setup_connection(&app);

    let missing_hydra_address =
        || Error::with_description("Missing Hydra Address", ErrorKind::ArgumentNotFound);
    let invalid_hydra_address =
        || Error::with_description("Invalid Hydra Address", ErrorKind::InvalidValue);
    let hydra_not_found =
        || Error::with_description("Hydra not found at address", ErrorKind::InvalidValue);
    let _hydra_mint_not_found =
        || Error::with_description("No Hydras for mint found", ErrorKind::InvalidValue);
    let hydra_mint_rpcs_error =
        |e: ClientError| Error::with_description(&*format!("{:?}", e), ErrorKind::InvalidValue);
    let get_hydra_account = |rpc: RpcClient| {
        move |hydra_pub: Pubkey| {
            let hpu = &hydra_pub;
            rpc.get_account_data(hpu)
                .map(|d| (hydra_pub, d))
                .map_err(|_| return hydra_not_found())
        }
    };

    let get_hydra_mints = |rpc: RpcClient| {
        move |hydra_pub: Pubkey, _fanout: Fanout| {
            rpc.get_program_accounts_with_config(
                &hydra::id(),
                RpcProgramAccountsConfig {
                    filters: Some(vec![RpcFilterType::Memcmp(Memcmp {
                        offset: 40,
                        bytes: MemcmpEncodedBytes::Base58(hydra_pub.to_string()),
                        encoding: None,
                    })]),
                    account_config: RpcAccountInfoConfig {
                        encoding: Some(UiAccountEncoding::Base64),
                        data_slice: None,
                        commitment: Some(CommitmentConfig {
                            commitment: CommitmentLevel::Confirmed,
                        }),
                    },
                    with_context: None,
                },
            )
            .map_err(|e| hydra_mint_rpcs_error(e))
            .map(|result| -> Vec<HydraMint> {
                result
                    .iter()
                    .map(|(addr, fanoutMintAccount)| HydraMint {
                        fanout_mint: FanoutMint::try_deserialize(
                            &mut fanoutMintAccount.data.as_slice(),
                        )
                        .unwrap(),
                        address: *addr,
                    })
                    .collect()
            })
        }
    };

    let parse_hydra_account = |input: (Pubkey, Vec<u8>)| -> Result<HydraObject, Error> {
        Fanout::try_deserialize(&mut input.1.as_slice())
            .map(|f| HydraObject {
                address: input.0,
                fanout: f,
                children: None,
            })
            .map_err(|_| invalid_hydra_address())
    };

    match app.subcommand() {
        (SHOW, Some(arg_matches)) => {
            println!("Running {}", SHOW);
            let hydra_pub = arg_matches
                .value_of("hydra_address")
                .ok_or(missing_hydra_address())
                .and_then(|hydra_address| {
                    Pubkey::from_str(hydra_address).map_err(|_| invalid_hydra_address())
                });
            let get_mints = get_hydra_mints(rpc);
            let get_h = get_hydra_account(rpc2);
            hydra_pub
                .and_then(get_h)
                .and_then(parse_hydra_account)
                .and_then(|hy| {
                    println!("{:#?}", hy);
                    get_mints(hy.address, hy.fanout)
                })
                .and_then(|mints| {
                    if mints.is_empty() {
                        println!("No Hydra Children");
                        return Ok(());
                    }
                    mints.iter().for_each(|m| {
                        println!("\n\n{:#?}", m);
                    });
                    return Ok(());
                })
                .map_err(|e| {
                    println!("{:?}", e);
                })
                .unwrap();
        }
        _ => unreachable!(),
    }
}
