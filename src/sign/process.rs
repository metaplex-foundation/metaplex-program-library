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
use mpl_token_metadata::{instruction::sign_metadata, ID as METAPLEX_PROGRAM_ID};
use retry::{delay::Exponential, retry};
use solana_client::rpc_client::RpcClient;
use solana_transaction_crawler::crawler::Crawler;
use tokio::sync::Semaphore;

use crate::{
    cache::load_cache,
    candy_machine::CANDY_MACHINE_ID,
    common::*,
    config::{Cluster, SugarConfig},
    pdas::{find_candy_machine_creator_pda, find_metadata_pda},
    setup::{get_rpc_url, setup_client, sugar_setup},
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

    let sugar_config = Arc::new(sugar_setup(args.keypair, args.rpc_url.clone())?);

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
        let rpc_url = get_rpc_url(args.rpc_url);

        let solana_cluster = if rpc_url.ends_with("8899") {
            Cluster::Localnet
        } else {
            solana_cluster
        };

        let account_keys = match solana_cluster {
            Cluster::Devnet | Cluster::Localnet => {
                let client = RpcClient::new(&rpc_url);
                let (creator, _) = find_candy_machine_creator_pda(&candy_machine_id);
                let creator = bs58::encode(creator).into_string();
                get_cm_creator_metadata_accounts(&client, &creator, 0)?
            }
            Cluster::Mainnet => {
                let client = RpcClient::new(&rpc_url);
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
            return Err(anyhow!("Not all NFTs were signed.".to_string()));
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
