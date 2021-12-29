use {
    clap::{crate_description, crate_name, crate_version, App, Arg, SubCommand},
    solana_clap_utils::{
        input_validators::{is_url_or_moniker, is_valid_signer, normalize_to_url_if_moniker},
        keypair::DefaultSigner,
    },
    solana_client::{rpc_client::RpcClient},
    solana_remote_wallet::remote_wallet::RemoteWalletManager,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        instruction::Instruction,
        message::Message,
        program_error::ProgramError,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        system_instruction,
        transaction::Transaction,
    },
    private_metadata::pod::*,
    private_metadata::encryption::{
        elgamal::*,
    },
    std::{convert::TryInto, process::exit, sync::Arc},
};

struct Config {
    commitment_config: CommitmentConfig,
    default_signer: Box<dyn Signer>,
    json_rpc_url: String,
    verbose: bool,
}

fn send(
    rpc_client: &RpcClient,
    msg: &str,
    instructions: &[Instruction],
    signers: &[&dyn Signer],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("==> {}", msg);
    let mut transaction =
        Transaction::new_unsigned(Message::new(instructions, Some(&signers[0].pubkey())));

    let (recent_blockhash, _fee_calculator) = rpc_client
        .get_recent_blockhash()
        .map_err(|err| format!("error: unable to get recent blockhash: {}", err))?;

    transaction
        .try_sign(&signers.to_vec(), recent_blockhash)
        .map_err(|err| format!("error: failed to sign transaction: {}", err))?;

    let signature = rpc_client
        .send_and_confirm_transaction_with_spinner(&transaction)
        .map_err(|err| format!("error: send transaction: {}", err))?;
    println!("Signature: {}", signature);
    Ok(())
}

fn specified_or_new(param: Option<String>) -> Keypair {
    param.map_or_else(
        || Keypair::new(),
        |s| Keypair::from_base58_string(&s),
    )
}

fn get_or_create_transfer_buffer(
    rpc_client: &RpcClient,
    payer: &dyn Signer,
    transfer_buffer: &dyn Signer,
    mint: &Pubkey,
    dest_elgamal: private_metadata::zk_token_elgamal::pod::ElGamalPubkey,
) -> Result<private_metadata::state::CipherKeyTransferBuffer, Box<dyn std::error::Error>> {
    let mut transfer_buffer_data = rpc_client.get_account_data(&transfer_buffer.pubkey());
    if transfer_buffer_data.is_err() {
        let buffer_len = private_metadata::state::CipherKeyTransferBuffer::get_packed_len();
        let buffer_minimum_balance_for_rent_exemption = rpc_client
            .get_minimum_balance_for_rent_exemption(buffer_len)?;

        send(
            rpc_client,
            &format!("Initializing transfer buffer"),
            &[
                system_instruction::create_account(
                    &payer.pubkey(),
                    &transfer_buffer.pubkey(),
                    buffer_minimum_balance_for_rent_exemption,
                    buffer_len as u64,
                    &private_metadata::ID,
                ),
                private_metadata::instruction::init_transfer(
                    payer.pubkey(),
                    *mint,
                    transfer_buffer.pubkey(),
                    dest_elgamal,
                ),
            ],
            &[payer, transfer_buffer],
        )?;

        transfer_buffer_data = rpc_client.get_account_data(&transfer_buffer.pubkey());
    } else {
        println!("Transfer buffer already initialized");
    }

    private_metadata::state::CipherKeyTransferBuffer::from_bytes(
        &transfer_buffer_data.unwrap())
        .map(|v| *v)  // seems a bit funky...
        .ok_or(Box::new(ProgramError::InvalidArgument))
}

fn ensure_create_instruction_buffer(
    rpc_client: &RpcClient,
    payer: &dyn Signer,
    instruction_buffer: &dyn Signer,
) -> Result<(), Box<dyn std::error::Error>> {
    use curve25519_dalek_onchain::instruction as dalek;
    let dsl = dalek::transer_proof_instructions(vec![3, 3, 5]);
    assert!(
        dsl == private_metadata::equality_proof::DSL_INSTRUCTION_BYTES,
        "DSL does not match!",
    );

    let instruction_buffer_len = (dalek::HEADER_SIZE + dsl.len()) as usize;

    let instruction_buffer_data = rpc_client.get_account_data(&instruction_buffer.pubkey());
    if let Ok(data) = instruction_buffer_data {
        assert!(data.len() >= instruction_buffer_len);
    } else {
        let txs = private_metadata::instruction::populate_transfer_proof_dsl(
            payer,
            instruction_buffer,
            |len| rpc_client.get_minimum_balance_for_rent_exemption(len).unwrap(),
        );

        for (i, tx) in txs.iter().enumerate() {
            send(
                rpc_client,
                &format!("Populating transfer proof DSL: {} of {}", i, txs.len()),
                tx.instructions.as_slice(),
                tx.signers.as_slice(),
            )?;
        }
    }

    Ok(())
}

fn ensure_buffers_closed(
    rpc_client: &RpcClient,
    payer: &dyn Signer,
    buffers: &[Pubkey],
) -> Result<(), Box<dyn std::error::Error>> {
    use curve25519_dalek_onchain::instruction as dalek;
    let mut instructions = vec![];
    for buffer in buffers {
        if let Ok(_) = rpc_client.get_account_data(&buffer) {
            instructions.push(
                dalek::close_buffer(
                    *buffer,
                    payer.pubkey(),
                ),
            );
        }
    }

    send(
        rpc_client,
        &format!("Closing input and compute buffers"),
        instructions.as_slice(),
        &[payer],
    )
}


struct ConfigureParams {
    mint: String,
    cipher_key: String,
    asset_url: String,
}

fn process_configure(
    rpc_client: &RpcClient,
    payer: &dyn Signer,
    _config: &Config,
    ConfigureParams{
        mint,
        cipher_key,
        asset_url,
    }: &ConfigureParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let mint = Pubkey::new(
        bs58::decode(&mint).into_vec()?.as_slice()
    );

    let elgamal_keypair = ElGamalKeypair::new(payer, &mint)?;
    let elgamal_pk = elgamal_keypair.public;

    let cipher_key_bytes = bs58::decode(&cipher_key).into_vec()?;
    let cipher_key_words = bytemuck::cast_slice::<u8, u32>(cipher_key_bytes.as_slice());

    let encrypted_cipher_key = cipher_key_words
        .iter()
        .map(|w| elgamal_pk.encrypt(*w).into())
        .collect::<Vec<_>>();

    let configure_metadata_ix = private_metadata::instruction::configure_metadata(
        payer.pubkey(),
        mint,
        elgamal_pk.into(),
        encrypted_cipher_key.as_slice(),
        asset_url.as_bytes(),
    );

    send(
        rpc_client,
        &format!("Configuring private metadata"),
        &[
            configure_metadata_ix,
        ],
        &[payer],
    )?;

    Ok(())
}

struct DecryptCipherKeyParams {
    mint: String,
}

async fn process_decrypt_cipher_key(
    rpc_client: &RpcClient,
    payer: &dyn Signer,
    _config: &Config,
    params: &DecryptCipherKeyParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let mint = Pubkey::new(
        bs58::decode(&params.mint).into_vec()?.as_slice()
    );

    let elgamal_keypair = ElGamalKeypair::new(payer, &mint)?;

    let private_metadata_key = private_metadata::instruction::get_private_metadata_address(&mint).0;
    let private_metadata_data = rpc_client.get_account_data(&private_metadata_key);

    if private_metadata_data.is_err() {
        eprintln!("error: no private metadata found for mint {}", params.mint);
        exit(1);
    }

    let private_metadata_data = private_metadata_data.unwrap();

    let private_metadata_account = private_metadata::state::PrivateMetadataAccount::from_bytes(
        &private_metadata_data).unwrap();

    let cipher_key_bytes = parallel_decrypt_cipher_key(
        &elgamal_keypair,
        &private_metadata_account,
    ).await?;

    let cipher_key = bs58::encode(cipher_key_bytes).into_string();
    println!("decrypted cipher key: {}", cipher_key);

    Ok(())
}

struct DecryptAssetParams {
    mint: String,
    out: String,
}

async fn process_decrypt_asset(
    rpc_client: &RpcClient,
    payer: &dyn Signer,
    _config: &Config,
    params: &DecryptAssetParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let mint = Pubkey::new(
        bs58::decode(&params.mint).into_vec()?.as_slice()
    );

    let elgamal_keypair = ElGamalKeypair::new(payer, &mint)?;

    let private_metadata_key = private_metadata::instruction::get_private_metadata_address(&mint).0;
    let private_metadata_data = rpc_client.get_account_data(&private_metadata_key);

    if private_metadata_data.is_err() {
        eprintln!("error: no private metadata found for mint {}", params.mint);
        exit(1);
    }

    let private_metadata_data = private_metadata_data.unwrap();

    let private_metadata_account = private_metadata::state::PrivateMetadataAccount::from_bytes(
        &private_metadata_data).unwrap();

    let private_asset_url = String::from_utf8(
        private_metadata_account.uri.0.to_vec())?
        .replace("\x00", "");

    let mut buf = Vec::new();
    let mut handle = curl::easy::Easy::new();
    handle.url(&private_asset_url).unwrap();
    handle.follow_location(true).unwrap(); // areweave has a few

    // something about mutable borrow in the lambda so open a scope
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|data| {
            buf.extend_from_slice(data);
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }

    println!("read {} bytes from {}", buf.len(), private_asset_url);

    let cipher = openssl::symm::Cipher::aes_192_cbc();

    let cipher_iv = &buf[..cipher.block_size()];
    let encrypted_bytes = &buf[cipher.block_size()..];

    let cipher_key_bytes = parallel_decrypt_cipher_key(
        &elgamal_keypair,
        private_metadata_account,
    ).await?;

    let decrypted = openssl::symm::decrypt(
        cipher,
        cipher_key_bytes.as_slice(),
        Some(cipher_iv),
        encrypted_bytes,
    ).unwrap();

    use std::io::Write;
    std::fs::File::create(params.out.clone())?
        .write_all(decrypted.as_slice())?;

    Ok(())
}

async fn parallel_decrypt_cipher_key(
    elgamal_keypair: &ElGamalKeypair,
    private_metadata_account: &private_metadata::state::PrivateMetadataAccount,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {

    let mut decrypt_tasks = vec![];
    for ct in private_metadata_account.encrypted_cipher_key {
        // TODO: clone
        let elgamal_secret = ElGamalSecretKey::from_bytes(elgamal_keypair.secret.to_bytes()).unwrap();
        decrypt_tasks.push(
            tokio::task::spawn(async move {
                let ct: ElGamalCiphertext = ct.try_into().ok()?;

                elgamal_secret.decrypt_u32(&ct)
            })
        );
    }

    let cipher_key_words = futures::future::join_all(decrypt_tasks).await;

    let cipher_key_words = cipher_key_words
        .iter()
        .map(|r| {
            match r {
                Ok(Some(w)) => {
                    *w
                }
                _ => {
                    // TODO: pass up
                    eprintln!("decrypt cipher text error");
                    exit(1);
                }
            }
        })
        .collect::<Vec<_>>();

    let cipher_key_bytes = bytemuck::cast_slice::<u32, u8>(cipher_key_words.as_slice());

    Ok(cipher_key_bytes.to_vec())
}

struct TransferParams {
    mint: String,
    recipient_pubkey: String,
    recipient_elgamal: String,
    transfer_buffer: Option<String>,
    instruction_buffer: Option<String>,
    input_buffer: Option<String>,
    compute_buffer: Option<String>,
}

async fn process_transfer(
    rpc_client: &RpcClient,
    payer: &dyn Signer,
    _config: &Config,
    params: &TransferParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let mint = Pubkey::new(
        bs58::decode(&params.mint).into_vec()?.as_slice()
    );

    let private_metadata_key = private_metadata::instruction::get_private_metadata_address(&mint).0;
    let private_metadata_data = rpc_client.get_account_data(&private_metadata_key);

    if private_metadata_data.is_err() {
        eprintln!("error: no private metadata found for mint {}", params.mint);
        exit(1);
    }

    let private_metadata_data = private_metadata_data.unwrap();
    let private_metadata_account = private_metadata::state::PrivateMetadataAccount::from_bytes(
        &private_metadata_data).unwrap();

    let transfer_buffer = specified_or_new(params.transfer_buffer.clone());
    let instruction_buffer = specified_or_new(params.instruction_buffer.clone());
    let input_buffer = specified_or_new(params.input_buffer.clone());
    let compute_buffer = specified_or_new(params.compute_buffer.clone());

    println!("Transfer buffer keypair: {}", transfer_buffer.to_base58_string());
    println!("Instruction buffer keypair: {}", instruction_buffer.to_base58_string());
    println!("Input buffer keypair: {}", input_buffer.to_base58_string());
    println!("Compute buffer keypair: {}", compute_buffer.to_base58_string());

    let elgamal_keypair = ElGamalKeypair::new(payer, &mint)?;
    let recipient_elgamal = ElGamalPubkey::from_bytes(
        base64::decode(&params.recipient_elgamal)?.as_slice().try_into()?
    ).ok_or(ProgramError::InvalidArgument)?;

    let transfer_buffer_account = get_or_create_transfer_buffer(
        rpc_client,
        payer,
        &transfer_buffer,
        &mint,
        recipient_elgamal.into(),
    )?;

    ensure_create_instruction_buffer(
        rpc_client,
        payer,
        &instruction_buffer,
    )?;

    // if previous run failed and we respecified...
    ensure_buffers_closed(
        rpc_client,
        payer,
        &[input_buffer.pubkey(), compute_buffer.pubkey()],
    )?;

    for chunk in 0..private_metadata::state::CIPHER_KEY_CHUNKS {
        let updated_mask = 1<<chunk;
        if transfer_buffer_account.updated & updated_mask == updated_mask {
            continue;
        }

        let ct = private_metadata_account.encrypted_cipher_key[chunk].try_into()?;
        let word = elgamal_keypair.secret.decrypt_u32(&ct).ok_or(ProgramError::InvalidArgument)?;

        let transfer = private_metadata::transfer_proof::TransferData::new(
            &elgamal_keypair,
            recipient_elgamal,
            word,
            ct,
        );

        let txs = private_metadata::instruction::transfer_chunk_slow_proof(
            &payer.pubkey(),
            &instruction_buffer.pubkey(),
            &input_buffer.pubkey(),
            &compute_buffer.pubkey(),
            &transfer,
            |len| rpc_client.get_minimum_balance_for_rent_exemption(len).unwrap(),
        );

        for (i, tx) in txs.iter().enumerate() {
            send(
                rpc_client,
                &format!("Building transfer chunk slow proof: {} of {}", i, txs.len()),
                tx.instructions.as_slice(),
                tx.signers
                    .as_slice()
                    .into_iter()
                    .map(|pk| -> Option<&dyn Signer> {
                        if *pk == payer.pubkey() {
                            Some(payer)
                        } else if *pk == instruction_buffer.pubkey() {
                            Some(&instruction_buffer)
                        } else if *pk == input_buffer.pubkey() {
                            Some(&input_buffer)
                        } else if *pk == compute_buffer.pubkey() {
                            Some(&compute_buffer)
                        } else {
                            None // shouldn't happen...
                        }
                    })
                    .collect::<Option<Vec<_>>>()
                    .unwrap()
                    .as_slice()
                    ,
            )?;
        }

        let transfer_chunk_ix = private_metadata::instruction::transfer_chunk_slow(
            payer.pubkey(),
            mint,
            transfer_buffer.pubkey(),
            instruction_buffer.pubkey(),
            input_buffer.pubkey(),
            compute_buffer.pubkey(),
            private_metadata::instruction::TransferChunkSlowData {
                chunk_idx: chunk as u8,
                transfer,
            },
        );

        send(
            rpc_client,
            &format!("Transferring chunk {}", chunk),
            &[
                transfer_chunk_ix,
            ],
            &[payer],
        )?;

        ensure_buffers_closed(
            rpc_client,
            payer,
            &[input_buffer.pubkey(), compute_buffer.pubkey()],
        )?;
    }

    use spl_associated_token_account::{
        create_associated_token_account,
        get_associated_token_address,
    };
    let payer_ata = get_associated_token_address(&payer.pubkey(), &mint);
    let recipient_pubkey  = Pubkey::new(
        bs58::decode(&params.recipient_pubkey).into_vec()?.as_slice()
    );
    let recipient_ata = get_associated_token_address(&recipient_pubkey, &mint);

    let mut instructions = vec![];

    if rpc_client.get_account_data(&recipient_ata).is_err() {
        instructions.push(
            create_associated_token_account(
                &payer.pubkey(),
                &recipient_pubkey,
                &mint,
            ),
        );
    }

    instructions.extend_from_slice(
        &[
            spl_token::instruction::transfer(
                &spl_token::id(),
                &payer_ata,
                &recipient_ata,
                &payer.pubkey(),
                &[],
                1,
            )?,
            private_metadata::instruction::fini_transfer(
                payer.pubkey(),
                mint,
                transfer_buffer.pubkey(),
            ),
        ],
    );

    send(
        rpc_client,
        &format!("Finalizing transfer"),
        instructions.as_slice(),
        &[payer],
    )?;

    Ok(())
}

struct ElGamalPubkeyParams {
    mint: String,
}

fn process_elgamal_pubkey(
    _rpc_client: &RpcClient,
    payer: &dyn Signer,
    _config: &Config,
    params: &ElGamalPubkeyParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let mint = Pubkey::new(
        bs58::decode(&params.mint).into_vec()?.as_slice()
    );
    println!("{}", ElGamalKeypair::new(payer, &mint)?.public);

    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transfer_buffer_param =
        Arg::with_name("transfer_buffer")
            .long("transfer_buffer")
            .value_name("KEYPAIR_STRING")
            .takes_value(true)
            .global(true)
            .help("Transfer buffer keypair to use (or create)");
    let instruction_buffer_param =
        Arg::with_name("instruction_buffer")
            .long("instruction_buffer")
            .value_name("KEYPAIR_STRING")
            .takes_value(true)
            .global(true)
            .help("Instruction buffer keypair to use (or create)");
    let input_buffer_param =
        Arg::with_name("input_buffer")
            .long("input_buffer")
            .value_name("KEYPAIR_STRING")
            .takes_value(true)
            .global(true)
            .help("Input buffer keypair to use (or create)");
    let compute_buffer_param =
        Arg::with_name("compute_buffer")
            .long("compute_buffer")
            .value_name("KEYPAIR_STRING")
            .takes_value(true)
            .global(true)
            .help("Compute buffer keypair to use (or create)");
    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .arg({
            let arg = Arg::with_name("config_file")
                .long("config")
                .value_name("PATH")
                .takes_value(true)
                .global(true)
                .help("Configuration file to use");
            if let Some(ref config_file) = *solana_cli_config::CONFIG_FILE {
                arg.default_value(config_file)
            } else {
                arg
            }
        })
        .arg(
            Arg::with_name("keypair")
                .long("keypair")
                .value_name("KEYPAIR")
                .validator(is_valid_signer)
                .takes_value(true)
                .global(true)
                .help("Filepath or URL to a keypair [default: client keypair]"),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .takes_value(false)
                .global(true)
                .help("Show additional information"),
        )
        .arg(
            Arg::with_name("json_rpc_url")
                .long("rpc_url")
                .value_name("URL")
                .takes_value(true)
                .global(true)
                .validator(is_url_or_moniker)
                .help("JSON RPC URL for the cluster [default: value from configuration file]"),
        )
        .subcommand(
            SubCommand::with_name("configure")
            .about("Configure private metadata")
            .arg(
                Arg::with_name("mint")
                    .long("mint")
                    .value_name("PUBKEY_STRING")
                    .takes_value(true)
                    .global(true)
                    .help("Mint of NFT to configure metadata for"),
            )
            .arg(
                Arg::with_name("asset_url")
                    .long("asset_url")
                    .value_name("URL")
                    .takes_value(true)
                    .global(true)
                    .help("URI of encrypted asset"),
            )
            .arg(
                Arg::with_name("cipher_key")
                    .long("cipher_key")
                    .value_name("STRING")
                    .takes_value(true)
                    .global(true)
                    .help("Base-58 encoded cipher key"),
            )
        )
        .subcommand(
            SubCommand::with_name("decrypt_key")
            .about("Decrypt cipher key associated with private metadata")
            .arg(
                Arg::with_name("mint")
                    .long("mint")
                    .value_name("PUBKEY_STRING")
                    .takes_value(true)
                    .global(true)
                    .help("Mint of NFT to decrypt cipher key of"),
            )
        )
        .subcommand(
            SubCommand::with_name("decrypt_asset")
            .about("Decrypt asset from private metadata")
            .arg(
                Arg::with_name("mint")
                    .long("mint")
                    .value_name("PUBKEY_STRING")
                    .takes_value(true)
                    .global(true)
                    .help("Mint of NFT to decrypt cipher key of"),
            )
            .arg(
                Arg::with_name("out")
                    .long("out")
                    .value_name("PATH")
                    .takes_value(true)
                    .global(true)
                    .help("Path to write the decrypted asset to"),
            )
        )
        .subcommand(
            SubCommand::with_name("transfer")
            .about("Transfer NFT and private metadata cipher key")
            .arg(
                Arg::with_name("mint")
                    .long("mint")
                    .value_name("PUBKEY_STRING")
                    .takes_value(true)
                    .global(true)
                    .help("NFT to transfer"),
            )
            .arg(
                Arg::with_name("recipient_pubkey")
                    .long("recipient_pubkey")
                    .value_name("PUBKEY_STRING")
                    .takes_value(true)
                    .global(true)
                    .help("Base-58 encoded public key of recipient"),
            )
            .arg(
                Arg::with_name("recipient_elgamal")
                    .long("recipient_elgamal")
                    .value_name("BASE64_STRING")
                    .takes_value(true)
                    .global(true)
                    .help("Base-64 encoded elgamal public key of recipient. The recipient can find this with the `elgamal_pubkey` instruction"),
            )
            .arg(transfer_buffer_param.clone())
            .arg(instruction_buffer_param.clone())
            .arg(input_buffer_param.clone())
            .arg(compute_buffer_param.clone())
        )
        .subcommand(
            SubCommand::with_name("elgamal_pubkey")
            .about("Print the elgamal pubkey associated with KEYPAIR and the mint")
            .arg(
                Arg::with_name("mint")
                    .long("mint")
                    .value_name("PUBKEY_STRING")
                    .takes_value(true)
                    .global(true)
                    .help("NFT to transfer"),
            )
        )
        .get_matches();

    let mut wallet_manager: Option<Arc<RemoteWalletManager>> = None;

    let config = {
        let cli_config = if let Some(config_file) = matches.value_of("config_file") {
            solana_cli_config::Config::load(config_file).unwrap_or_default()
        } else {
            solana_cli_config::Config::default()
        };

        let default_signer = DefaultSigner::new(
            "keypair",
            matches
                .value_of(&"keypair")
                .map(|s| s.to_string())
                .unwrap_or_else(|| cli_config.keypair_path.clone()),
        );

        Config {
            json_rpc_url: normalize_to_url_if_moniker(
                matches
                    .value_of("json_rpc_url")
                    .unwrap_or(&cli_config.json_rpc_url)
                    .to_string(),
            ),
            default_signer: default_signer
                .signer_from_path(&matches, &mut wallet_manager)
                .unwrap_or_else(|err| {
                    eprintln!("error: {}", err);
                    exit(1);
                }),
            verbose: matches.is_present("verbose"),
            commitment_config: CommitmentConfig::confirmed(),
        }
    };
    solana_logger::setup_with_default("solana=info");

    if config.verbose {
        println!("JSON RPC URL: {}", config.json_rpc_url);
    }
    let rpc_client =
        RpcClient::new_with_commitment(config.json_rpc_url.clone(), config.commitment_config);

    match matches.subcommand() {
        ("configure", Some(sub_m)) => {
            process_configure(
                &rpc_client,
                config.default_signer.as_ref(),
                &config,
                &ConfigureParams {
                    mint: sub_m.value_of("mint").unwrap().to_string(),
                    cipher_key: sub_m.value_of("cipher_key").unwrap().to_string(),
                    asset_url: sub_m.value_of("asset_url").unwrap().to_string(),
                },
            ).unwrap_or_else(|err| {
                eprintln!("error: {}", err);
                exit(1);
            });
        }
        ("decrypt_key", Some(sub_m)) => {
            process_decrypt_cipher_key(
                &rpc_client,
                config.default_signer.as_ref(),
                &config,
                &DecryptCipherKeyParams {
                    mint: sub_m.value_of("mint").unwrap().to_string(),
                },
            ).await.unwrap_or_else(|err| {
                eprintln!("error: {}", err);
                exit(1);
            });
        }
        ("decrypt_asset", Some(sub_m)) => {
            process_decrypt_asset(
                &rpc_client,
                config.default_signer.as_ref(),
                &config,
                &DecryptAssetParams {
                    mint: sub_m.value_of("mint").unwrap().to_string(),
                    out: sub_m.value_of("out").unwrap().to_string(),
                },
            ).await.unwrap_or_else(|err| {
                eprintln!("error: {}", err);
                exit(1);
            });
        }
        ("transfer", Some(sub_m)) => {
            process_transfer(
                &rpc_client,
                config.default_signer.as_ref(),
                &config,
                &TransferParams {
                    mint: sub_m.value_of("mint").unwrap().to_string(),
                    recipient_pubkey: sub_m.value_of("recipient_pubkey").unwrap().to_string(),
                    recipient_elgamal: sub_m.value_of("recipient_elgamal").unwrap().to_string(),
                    transfer_buffer: sub_m.value_of("transfer_buffer").map(|s| s.into()),
                    instruction_buffer: sub_m.value_of("instruction_buffer").map(|s| s.into()),
                    input_buffer: sub_m.value_of("input_buffer").map(|s| s.into()),
                    compute_buffer: sub_m.value_of("compute_buffer").map(|s| s.into()),
                },
            ).await.unwrap_or_else(|err| {
                eprintln!("error: {}", err);
                exit(1);
            });
        }
        ("elgamal_pubkey", Some(sub_m)) => {
            process_elgamal_pubkey(
                &rpc_client,
                config.default_signer.as_ref(),
                &config,
                &ElGamalPubkeyParams {
                    mint: sub_m.value_of("mint").unwrap().to_string(),
                },
            ).unwrap_or_else(|err| {
                eprintln!("error: {}", err);
                exit(1);
            });
        }
        _ => {
        }
    }

    Ok(())
}
