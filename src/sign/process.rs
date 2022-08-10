use std::{str::FromStr, sync::Arc};

pub use anchor_client::{
    solana_sdk::{
        account::Account,
        commitment_config::{CommitmentConfig, CommitmentLevel},
        native_token::LAMPORTS_PER_SOL,
        pubkey::Pubkey,
        signature::{Keypair, Signature, Signer},
        system_instruction, system_program, sysvar,
        transaction::Transaction,
    },
    Client, Program,
};
use anyhow::Error;
use console::style;
use mpl_token_metadata::{
    instruction::sign_metadata, ID as TOKEN_METADATA_PROGRAM_ID, ID as METAPLEX_PROGRAM_ID,
};
use retry::{delay::Exponential, retry};
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_transaction_crawler::crawler::Crawler;
use tokio::sync::Semaphore;

use crate::{
    cache::load_cache,
    candy_machine::CANDY_MACHINE_ID,
    common::*,
    config::{Cluster, SugarConfig},
    pdas::{find_candy_machine_creator_pda, find_metadata_pda},
    setup::{setup_client, sugar_setup},
    utils::*,
};

pub struct SignArgs {
    pub candy_machine_id: Option<String>,
    pub keypair: Option<String>,
    pub cache: String,
    pub rpc_url: Option<String>,
    pub mint: Option<String>,
}

pub async fn process_sign(args: SignArgs) -> Result<()> {
    // (1) Setting up connection
    println!(
        "{} {}Initializing connection",
        if args.mint.is_some() {
            style("[1/2]").bold().dim()
        } else {
            style("[1/3]").bold().dim()
        },
        COMPUTER_EMOJI
    );

    let pb = spinner_with_style();
    pb.set_message("Connecting...");

    let sugar_config = Arc::new(sugar_setup(args.keypair, args.rpc_url)?);

    let client = setup_client(&sugar_config)?;
    let program = client.program(CANDY_MACHINE_ID);

    pb.finish_with_message("Connected");

    if let Some(mint_id) = args.mint {
        println!(
            "\n{} {}Signing one NFT",
            style("[2/2]").bold().dim(),
            SIGNING_EMOJI,
        );
        let pb = spinner_with_style();
        pb.set_message(format!("Signing NFT with mint id {}.", mint_id));

        let account_pubkey = Pubkey::from_str(&mint_id)?;
        let metadata_pubkey = find_metadata_pda(&account_pubkey);
        match sign(Arc::clone(&sugar_config.clone()), metadata_pubkey).await {
            Ok(signature) => format!("{} {:?}", style("Signature:").bold(), signature),
            Err(err) => {
                pb.abandon_with_message(format!("{}", style("Signing failed ").red().bold()));
                error!("{:?}", err);
                return Err(err);
            }
        };

        pb.finish();
    } else {
        println!(
            "\n{} {}Fetching mint ids",
            style("[2/3]").bold().dim(),
            LOOKING_GLASS_EMOJI,
        );

        let mut errors = Vec::new();

        let candy_machine_id = match args.candy_machine_id {
            Some(candy_machine_id) => candy_machine_id,
            None => {
                let cache = load_cache(&args.cache, false)?;
                cache.program.candy_machine
            }
        };

        let candy_machine_id = Pubkey::from_str(&candy_machine_id)
            .expect("Failed to parse pubkey from candy machine id.");

        let solana_cluster: Cluster = get_cluster(program.rpc())?;
        let account_keys = match solana_cluster {
            Cluster::Devnet => {
                let client = RpcClient::new("https://devnet.genesysgo.net/");
                let (creator, _) = find_candy_machine_creator_pda(&candy_machine_id);
                let creator = bs58::encode(creator).into_string();
                get_cm_creator_accounts(&client, &creator, 0)?
            }
            Cluster::Mainnet => {
                let client = RpcClient::new("https://ssc-dao.genesysgo.net");
                let crawled_accounts = Crawler::get_cmv2_mints(client, candy_machine_id).await?;
                match crawled_accounts.get("metadata") {
                    Some(accounts) => accounts
                        .iter()
                        .map(|account| Pubkey::from_str(account).unwrap())
                        .collect::<Vec<Pubkey>>(),
                    None => Vec::new(),
                }
            }
            _ => {
                return Err(anyhow!(
                    "Cluster being used is unsupported for this command."
                ))
            }
        };

        if account_keys.is_empty() {
            pb.finish_with_message(format!("{}", style("No NFTs found.").green().bold()));
            return Err(anyhow!(format!(
                "No NFTs found for candy machine id {candy_machine_id}.",
            )));
        } else {
            pb.finish_with_message(format!("Found {:?} accounts", account_keys.len() as u64));
            println!(
                "\n{} {}Signing mint accounts",
                style("[3/3]").bold().dim(),
                SIGNING_EMOJI
            );
        }

        let pb = progress_bar_with_style(account_keys.len() as u64);

        let semaphore = Arc::new(Semaphore::new(100));
        let mut join_handles = Vec::new();
        for account in account_keys {
            let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();
            let config = sugar_config.clone();
            let pb = pb.clone();

            join_handles.push(tokio::spawn(async move {
                let _permit = permit;
                sign(Arc::clone(&config), account).await.ok();
                pb.inc(1);
            }));
        }

        for handle in join_handles {
            handle.await.map_err(|err| errors.push(err)).ok();
        }

        if !errors.is_empty() {
            pb.abandon_with_message(format!("{}", style("Signing command failed ").red().bold()));
            return Err(anyhow!(format!("Not all NFTs were signed.")));
        } else {
            pb.finish_with_message(format!(
                "{}",
                style("All NFTs signed successfully.").green().bold()
            ));
        }
    }

    Ok(())
}

async fn sign(config: Arc<SugarConfig>, metadata: Pubkey) -> Result<(), Error> {
    let client = setup_client(&config)?;
    let program = client.program(CANDY_MACHINE_ID);

    let recent_blockhash = program.rpc().get_latest_blockhash()?;

    let ix = sign_metadata(METAPLEX_PROGRAM_ID, metadata, config.keypair.pubkey());
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&config.keypair.pubkey()),
        &[&config.keypair],
        recent_blockhash,
    );

    // Send tx with retries.
    retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || program.rpc().send_and_confirm_transaction(&tx),
    )?;

    Ok(())
}

pub fn get_cm_creator_accounts(
    client: &RpcClient,
    creator: &str,
    position: usize,
) -> Result<Vec<Pubkey>> {
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
        },
        with_context: None,
    };

    let accounts = client
        .get_program_accounts_with_config(&TOKEN_METADATA_PROGRAM_ID, config)?
        .into_iter()
        .map(|(pubkey, _account)| pubkey)
        .collect::<Vec<Pubkey>>();

    Ok(accounts)
}
