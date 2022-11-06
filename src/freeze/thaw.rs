use std::time::Duration;

use super::*;

pub struct ThawArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub config: String,
    pub all: bool,
    pub nft_mint: Option<String>,
    pub candy_machine: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FailedThaw {
    nft: ThawNft,
    error: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ThawNft {
    #[serde(serialize_with = "serialize_pubkey")]
    mint: Pubkey,
    #[serde(serialize_with = "serialize_pubkey")]
    owner: Pubkey,
    #[serde(serialize_with = "serialize_pubkey")]
    token_account: Pubkey,
}

fn serialize_pubkey<S>(p: &Pubkey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    p.to_string().serialize(serializer)
}

#[derive(Debug, Deserialize)]
pub struct JRpcResponse {
    value: Vec<TokenAccount>,
}

#[derive(Debug, Deserialize)]
struct TokenAccount {
    address: String,
    amount: String,
    // decimals: u8,
    // #[serde(rename = "uiAmount")]
    // ui_amount: f32,
    // #[serde(rename = "uiAmountString")]
    // ui_amount_string: String,
}

pub async fn process_thaw(args: ThawArgs) -> Result<()> {
    let sugar_config = sugar_setup(args.keypair.clone(), args.rpc_url.clone())?;
    let client = setup_client(&sugar_config)?;
    let program = client.program(CANDY_MACHINE_ID);
    let rpc_url = get_rpc_url(args.rpc_url.clone());
    let rpc_client = RpcClient::new(&rpc_url);

    // The candy machine id specified takes precedence over the one from the cache.
    let candy_machine_id = match args.candy_machine {
        Some(ref candy_machine_id) => candy_machine_id.to_owned(),
        None => {
            let cache = load_cache(&args.cache, false)?;
            cache.program.candy_machine
        }
    };

    let candy_pubkey = Pubkey::from_str(&candy_machine_id)
        .map_err(|_| anyhow!("Failed to parse candy machine id: {}", &candy_machine_id))?;

    let total_steps = if args.all { 4 } else { 2 };

    println!(
        "{} {}Loading candy machine",
        style(format!("[1/{}]", total_steps)).bold().dim(),
        LOOKING_GLASS_EMOJI
    );
    println!("{} {}", style("Candy machine ID:").bold(), candy_machine_id);

    let pb = spinner_with_style();
    pb.set_message("Connecting...");
    let _candy_machine_state =
        get_candy_machine_state(&sugar_config, &Pubkey::from_str(&candy_machine_id)?)?;

    pb.finish_with_message("Done");

    if !args.all {
        println!(
            "\n{} {}Thawing single NFT. . .",
            style(format!("[2/{}]", total_steps)).bold().dim(),
            MONEY_BAG_EMOJI
        );

        let nft_mint = if let Some(nft_mint) = &args.nft_mint {
            nft_mint.to_owned()
        } else {
            return Err(anyhow!("NFT mint is required if thawing a single NFT"));
        };

        let nft_mint_pubkey = Pubkey::from_str(&nft_mint)
            .map_err(|_| anyhow!("Failed to parse nft mint id: {}", &nft_mint))?;

        let config = Arc::new(sugar_config);

        let request = RpcRequest::Custom {
            method: "getTokenLargestAccounts",
        };
        let params = json!([nft_mint, { "commitment": "confirmed" }]);
        let result: JRpcResponse = rpc_client.send(request, params).unwrap();

        let token_accounts: Vec<TokenAccount> = result
            .value
            .into_iter()
            .filter(|account| account.amount.parse::<u64>().unwrap() == 1)
            .collect();

        if token_accounts.len() > 1 {
            return Err(anyhow!(
                "Mint account {} had more than one token account with 1 token",
                nft_mint
            ));
        }

        if token_accounts.is_empty() {
            return Err(anyhow!(
                "Mint account {} had zero token accounts with 1 token",
                nft_mint
            ));
        }

        let token_account = Pubkey::from_str(&token_accounts[0].address).unwrap();

        let account = program
            .rpc()
            .get_account_with_commitment(&token_account, CommitmentConfig::confirmed())
            .unwrap()
            .value
            .unwrap();
        let account_data = SplAccount::unpack(&account.data).unwrap();
        let owner = account_data.owner;

        // Only thaw frozen accounts.
        if !account_data.is_frozen() {
            println!("\n NFT is already thawed!");
            return Ok(());
        }

        let nft = ThawNft {
            mint: nft_mint_pubkey,
            owner,
            token_account,
        };

        let pb = spinner_with_style();
        pb.set_message("Sending thaw transaction...");

        let signature = thaw_nft(config, &candy_pubkey, &nft)?;

        pb.finish_with_message(format!(
            "{} {}",
            style("Thaw NFT signature:").bold(),
            signature
        ));
        return Ok(());
    }

    // Thaw all frozen NFTs.
    println!(
        "\n{} {}Getting minted NFTs for candy machine {}",
        style(format!("[2/{}]", total_steps)).bold().dim(),
        LOOKING_GLASS_EMOJI,
        candy_machine_id
    );

    let pb = spinner_with_style();
    pb.set_message("Searching...");

    let solana_cluster: Cluster = get_cluster(program.rpc())?;
    let rpc_url = get_rpc_url(args.rpc_url);
    let client = RpcClient::new_with_timeout(&rpc_url, Duration::from_secs(300));

    let solana_cluster = if rpc_url.ends_with("8899") {
        Cluster::Localnet
    } else {
        solana_cluster
    };

    let mint_pubkeys = match solana_cluster {
        Cluster::Devnet | Cluster::Localnet | Cluster::Mainnet => {
            let (creator, _) = find_candy_machine_creator_pda(&candy_pubkey);
            let creator = bs58::encode(creator).into_string();
            get_cm_creator_mint_accounts(&client, &creator, 0)?
        }
        _ => {
            return Err(anyhow!(
                "Cluster being used is unsupported for this command."
            ))
        }
    };

    if mint_pubkeys.is_empty() {
        pb.finish_with_message(format!("{}", style("No NFTs found.").green().bold()));
        return Err(anyhow!(format!(
            "No NFTs found for candy machine id {candy_machine_id}.",
        )));
    } else {
        pb.finish_with_message(format!("Found {:?} accounts", mint_pubkeys.len() as u64));
    }

    let pb = progress_bar_with_style(mint_pubkeys.len() as u64);
    pb.set_message("Getting NFT information....");

    let semaphore = Arc::new(Semaphore::new(100));
    let client = Arc::new(client);

    let mut tasks = Vec::new();
    let mut thaw_tasks = Vec::new();
    let errors = Arc::new(Mutex::new(Vec::new()));
    let thaw_errors = Arc::new(Mutex::new(Vec::new()));
    let thaw_nfts = Arc::new(Mutex::new(Vec::new()));
    let failed_thaws = Arc::new(Mutex::new(Vec::new()));

    let mint_pubkeys_len = mint_pubkeys.len();

    for mint in mint_pubkeys {
        let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();
        let client = client.clone();
        let pb = pb.clone();
        let errors = errors.clone();
        let thaw_nfts = thaw_nfts.clone();

        tasks.push(tokio::spawn(async move {
            let _permit = permit;

            let request = RpcRequest::Custom {
                method: "getTokenLargestAccounts",
            };
            let params = json!([mint.to_string(), { "commitment": "confirmed" }]);
            let result: JRpcResponse = client.send(request, params).unwrap();

            let token_accounts: Vec<TokenAccount> = result
                .value
                .into_iter()
                .filter(|account| account.amount.parse::<u64>().unwrap() == 1)
                .collect();

            if token_accounts.len() != 1 {
                errors.lock().unwrap().push(anyhow!(
                    "Mint account {} had more than one token account with 1 token",
                    mint
                ));
                return;
            }

            let token_account = Pubkey::from_str(&token_accounts[0].address).unwrap();
            let account = client
                .get_account_with_commitment(&token_account, CommitmentConfig::confirmed())
                .unwrap()
                .value
                .unwrap();
            let account_data = SplAccount::unpack(&account.data).unwrap();
            let owner = account_data.owner;

            // Only thaw frozen accounts.
            if account_data.is_frozen() {
                thaw_nfts.lock().unwrap().push(ThawNft {
                    mint,
                    token_account,
                    owner,
                });

                pb.inc(1);
            }
        }));
    }

    for task in tasks {
        task.await
            .map_err(|err| errors.lock().unwrap().push(anyhow!(err)))
            .ok();
    }

    if !errors.lock().unwrap().is_empty() {
        println!(
            "{} {}/{} {}",
            style("Found :").bold(),
            errors.lock().unwrap().len(),
            mint_pubkeys_len,
            style("NFT information").bold()
        );
    }

    pb.finish_with_message(format!(
        "{}",
        style("Finished fetching NFT information.").green().bold()
    ));

    let config = Arc::new(sugar_config);

    let nfts = thaw_nfts.lock().unwrap().clone();
    let thaw_pb = progress_bar_with_style(nfts.len() as u64);
    thaw_pb.set_message("Thawing NFTs....");

    for nft in nfts.into_iter() {
        let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();
        let thaw_pb = thaw_pb.clone();
        let failed_thaws = failed_thaws.clone();

        let config = config.clone();

        thaw_tasks.push(tokio::spawn(async move {
            let _permit = permit;

            let _signature = thaw_nft(config, &candy_pubkey, &nft).map_err(|e| {
                failed_thaws.lock().unwrap().push(FailedThaw {
                    nft: nft.clone(),
                    error: e.to_string(),
                });
            });

            thaw_pb.inc(1);
        }));
    }

    for task in thaw_tasks {
        match task.await {
            Ok(_) => {}
            Err(err) => thaw_errors.lock().unwrap().push(anyhow!(err)),
        }
    }

    if !thaw_errors.lock().unwrap().is_empty() || !failed_thaws.lock().unwrap().is_empty() {
        thaw_pb.abandon_with_message(format!(
            "{}",
            style("Failed to Thaw all NFTs.").red().bold()
        ));
        let failed_thaws = Arc::try_unwrap(failed_thaws).unwrap().into_inner().unwrap();

        let failed_thaws_cache = File::create("failed_thaws.json")?;
        serde_json::to_writer(failed_thaws_cache, &failed_thaws)?;

        return Err(anyhow!("Not all NFTs were thawed.".to_string()));
    } else {
        thaw_pb.finish_with_message(format!(
            "{}",
            style("All NFTs thawed successfully.").green().bold()
        ));
    }

    let remaining_nfts = Arc::try_unwrap(failed_thaws).unwrap().into_inner().unwrap();

    let remaining_items_cache = File::create("remaining_thaw_items_cache.json")?;
    serde_json::to_writer_pretty(remaining_items_cache, &remaining_nfts)?;

    Ok(())
}

fn thaw_nft(
    config: Arc<SugarConfig>,
    candy_machine_id: &Pubkey,
    nft: &ThawNft,
) -> Result<Signature> {
    let client = setup_client(&config)?;
    let program = client.program(CANDY_MACHINE_ID);

    let (freeze_pda, _) = find_freeze_pda(candy_machine_id);
    let edition = find_master_edition_pda(&nft.mint);

    let builder = program
        .request()
        .accounts(nft_accounts::ThawNFT {
            freeze_pda,
            candy_machine: *candy_machine_id,
            token_account: nft.token_account,
            owner: nft.owner,
            mint: nft.mint,
            edition,
            payer: program.payer(),
            token_program: spl_token::ID,
            token_metadata_program: mpl_token_metadata::ID,
            system_program: system_program::ID,
        })
        .args(nft_instruction::ThawNft);

    let sig = builder.send()?;

    Ok(sig)
}
