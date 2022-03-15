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
        transaction::Transaction,
    },
    stealth::pod::*,
    stealth::encryption::{
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

    let recent_blockhash = rpc_client
        .get_latest_blockhash()
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
    transfer_buffer_key: &Pubkey,
    mint: &Pubkey,
    recipient: &Pubkey,
) -> Result<stealth::state::CipherKeyTransferBuffer, Box<dyn std::error::Error>> {
    let mut transfer_buffer_data = rpc_client.get_account_data(transfer_buffer_key);
    if transfer_buffer_data.is_err() {
        send(
            rpc_client,
            &format!("Initializing transfer buffer"),
            &[
                stealth::instruction::init_transfer(
                    &payer.pubkey(),
                    mint,
                    recipient,
                ),
            ],
            &[payer],
        )?;

        transfer_buffer_data = rpc_client.get_account_data(transfer_buffer_key);
    } else {
        println!("Transfer buffer already initialized");
    }

    stealth::state::CipherKeyTransferBuffer::from_bytes(
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
    let dsl = dalek::transfer_proof_instructions(vec![11], false);
    assert!(
        dsl == stealth::equality_proof::DSL_INSTRUCTION_BYTES,
        "DSL does not match!",
    );

    let instruction_buffer_len = (dalek::HEADER_SIZE + dsl.len()) as usize;

    let instruction_buffer_data = rpc_client.get_account_data(&instruction_buffer.pubkey());
    if let Ok(data) = instruction_buffer_data {
        assert_eq!(data.len(), instruction_buffer_len);
        assert_eq!(&data[dalek::HEADER_SIZE..], &dsl);
        println!("Instruction buffer {} already matches", instruction_buffer.pubkey());
    } else {
        println!("Populating instruction buffer {}", instruction_buffer.pubkey());
        let txs = stealth::instruction::populate_transfer_proof_dsl(
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

    if instructions.len() == 0 {
        return Ok(());
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
    let encrypted_cipher_key = elgamal_pk.encrypt(
        CipherKey(cipher_key_bytes.as_slice().try_into()?)
    ).into();

    let configure_metadata_ix = stealth::instruction::configure_metadata(
        payer.pubkey(),
        mint,
        elgamal_pk.into(),
        &encrypted_cipher_key,
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

    let stealth_key = stealth::instruction::get_stealth_address(&mint).0;
    let stealth_data = rpc_client.get_account_data(&stealth_key);

    if stealth_data.is_err() {
        eprintln!("error: no private metadata found for mint {}", params.mint);
        exit(1);
    }

    let stealth_data = stealth_data.unwrap();

    let stealth_account = stealth::state::StealthAccount::from_bytes(
        &stealth_data).unwrap();

    let cipher_key_bytes = parallel_decrypt_cipher_key(
        &elgamal_keypair,
        &stealth_account,
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

    let stealth_key = stealth::instruction::get_stealth_address(&mint).0;
    let stealth_data = rpc_client.get_account_data(&stealth_key);

    if stealth_data.is_err() {
        eprintln!("error: no private metadata found for mint {}", params.mint);
        exit(1);
    }

    let stealth_data = stealth_data.unwrap();

    let stealth_account = stealth::state::StealthAccount::from_bytes(
        &stealth_data).unwrap();

    let private_asset_url = String::from_utf8(
        stealth_account.uri.0.to_vec())?
        .replace("\x00", "");

    let mut handle = curl::easy::Easy::new();
    handle.follow_location(true).unwrap(); // areweave has a few

    // mut so that we can reset handle.url
    let mut read_all = |url: &str| -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        handle.url(url)?;
        let mut buf = vec![];

        // something about mutable borrow in the lambda so open a scope
        {
            let mut transfer = handle.transfer();
            transfer.write_function(|data| {
                buf.extend_from_slice(data);
                Ok(data.len())
            })?;
            transfer.perform()?;
        }

        println!("read {} bytes from {}", buf.len(), url);

        Ok(buf)
    };

    let manifest_buf = read_all(&private_asset_url)?;

    let manifest: serde_json::Value = serde_json::from_str(
        std::str::from_utf8(manifest_buf.as_slice())?
    )?;

    let out_dir = std::path::Path::new(&params.out);
    std::fs::create_dir_all(out_dir)?;

    let cipher = openssl::symm::Cipher::aes_192_cbc();
    let cipher_key_bytes = parallel_decrypt_cipher_key(
        &elgamal_keypair,
        stealth_account,
    ).await?;

    let url_re = regex::Regex::new(r"https://www.arweave.net/(.*)")?;

    let invalid_manifest = "Invalid Manifest";
    for asset in manifest["encrypted_assets"].as_array().ok_or(invalid_manifest)? {
        let asset_url = asset["uri"].as_str().ok_or(invalid_manifest)?;
        let basename = url_re
            .captures(asset_url).ok_or(invalid_manifest)?
            // firsrt capture group that is not the whole match
            .get(1).ok_or(invalid_manifest)?
            .as_str();

        let encrypted_buf = read_all(&asset_url)?;
        let cipher_iv = &encrypted_buf[..cipher.block_size()];
        let encrypted_bytes = &encrypted_buf[cipher.block_size()..];

        let decrypted = openssl::symm::decrypt(
            cipher,
            cipher_key_bytes.as_slice(),
            Some(cipher_iv),
            encrypted_bytes,
        )?;

        use std::io::Write;

        std::fs::File::create(out_dir.join(basename))?
            .write_all(decrypted.as_slice())?;
    }

    Ok(())
}

async fn parallel_decrypt_cipher_key(
    elgamal_keypair: &ElGamalKeypair,
    stealth_account: &stealth::state::StealthAccount,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let cipher_key = elgamal_keypair.secret.decrypt(
        &stealth_account.encrypted_cipher_key.try_into()?
    )?;

    Ok(cipher_key.0.to_vec())
}

struct TransferParams {
    mint: String,
    recipient_pubkey: String,
    instruction_buffer: String,
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

    let stealth_key = stealth::instruction::get_stealth_address(&mint).0;
    let stealth_data = rpc_client.get_account_data(&stealth_key);

    if stealth_data.is_err() {
        eprintln!("error: no private metadata found for mint {}", params.mint);
        exit(1);
    }

    let stealth_data = stealth_data.unwrap();
    let stealth_account = stealth::state::StealthAccount::from_bytes(
        &stealth_data).unwrap();

    let instruction_buffer = Pubkey::new(
        bs58::decode(&params.instruction_buffer).into_vec()?.as_slice()
    );

    if rpc_client.get_account_data(&instruction_buffer).is_err() {
        eprintln!("error: instruction buffer not populated");
        exit(1);
    }

    let input_buffer = specified_or_new(params.input_buffer.clone());
    let compute_buffer = specified_or_new(params.compute_buffer.clone());

    println!("Input buffer keypair: {}", input_buffer.to_base58_string());
    println!("Compute buffer keypair: {}", compute_buffer.to_base58_string());

    let elgamal_keypair = ElGamalKeypair::new(payer, &mint)?;
    let recipient_pubkey = Pubkey::new(
        bs58::decode(&params.recipient_pubkey).into_vec()?.as_slice()
    );

    let recipient_elgamal: stealth::encryption::elgamal::ElGamalPubkey = stealth::state::EncryptionKeyBuffer::from_bytes(
        rpc_client.get_account_data(
            &stealth::instruction::get_elgamal_pubkey_address(
                &recipient_pubkey,
                &mint,
            ).0,
        )?.as_slice()
    ).unwrap().elgamal_pk.try_into()?;

    let transfer_buffer_key = stealth::instruction::get_transfer_buffer_address(
        &recipient_pubkey, &mint).0;

    let transfer_buffer_account = get_or_create_transfer_buffer(
        rpc_client,
        payer,
        &transfer_buffer_key,
        &mint,
        &recipient_pubkey,
    )?;

    // if previous run failed and we respecified...
    ensure_buffers_closed(
        rpc_client,
        payer,
        &[input_buffer.pubkey(), compute_buffer.pubkey()],
    )?;

    if !bool::from(&transfer_buffer_account.updated) {
        let ct = stealth_account.encrypted_cipher_key.try_into()?;
        let word = elgamal_keypair.secret.decrypt(&ct)?;

        let transfer = stealth::transfer_proof::TransferData::new(
            &elgamal_keypair,
            recipient_elgamal,
            word,
            ct,
        );

        let txs = stealth::instruction::transfer_chunk_slow_proof(
            &payer.pubkey(),
            &instruction_buffer,
            &input_buffer.pubkey(),
            &compute_buffer.pubkey(),
            &transfer,
            |len| rpc_client.get_minimum_balance_for_rent_exemption(len).unwrap(),
        )?;

        let signers_to_kps = |signers: &[Pubkey]| {
            signers
                .into_iter()
                .map(|pk| -> Option<&dyn Signer> {
                    if *pk == payer.pubkey() {
                        Some(payer)
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
        };

        // first setup serially
        let setup_count = 2;
        for (i, tx) in txs[..setup_count].iter().enumerate() {
            send(
                rpc_client,
                &format!("Setting up crank: {} of {}", i + 1, setup_count),
                tx.instructions.as_slice(),
                &signers_to_kps(&tx.signers),
            )?;
        }

        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .map_err(|err| format!("error: unable to get recent blockhash: {}", err))?;

        let mut signatures = vec![];
        for (i, tx) in txs[setup_count..].iter().enumerate() {
            let instructions = &tx.instructions;
            let signers = &signers_to_kps(&tx.signers);
            let mut transaction =
                Transaction::new_unsigned(Message::new(instructions, Some(&signers[0].pubkey())));

            transaction
                .try_sign(&signers.to_vec(), recent_blockhash)
                .map_err(|err| format!("error: failed to sign transaction: {}", err))?;

            let signature = rpc_client
                .send_transaction(&transaction)
                .map_err(|err| format!("error: send transaction: {}", err))?;
            println!("Signature {}: {}", i + 1, signature);

            signatures.push(signature);
        }

        loop {
            let statuses = rpc_client.get_signature_statuses(&signatures)?.value;
            let confirmed = statuses.iter().filter(|s| {
                if let Some(status) = s {
                    return status.confirmation_status == Some(
                        solana_transaction_status::TransactionConfirmationStatus::Confirmed
                    );
                }
                return false;
            }).count();
            println!("Confirmed: {} of {}", confirmed, signatures.len());
            if confirmed == signatures.len() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        };

        let transfer_ix = stealth::instruction::transfer_chunk_slow(
            payer.pubkey(),
            mint,
            transfer_buffer_key,
            instruction_buffer,
            input_buffer.pubkey(),
            compute_buffer.pubkey(),
            stealth::instruction::TransferChunkSlowData {
                transfer,
            },
        );

        send(
            rpc_client,
            &format!("Transferring data"),
            &[
                transfer_ix,
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
            stealth::instruction::fini_transfer(
                payer.pubkey(),
                mint,
                transfer_buffer_key,
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

fn process_publish_elgamal_pubkey(
    rpc_client: &RpcClient,
    payer: &dyn Signer,
    _config: &Config,
    params: &ElGamalPubkeyParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let mint = Pubkey::new(
        bs58::decode(&params.mint).into_vec()?.as_slice()
    );

    let elgamal_pk = ElGamalKeypair::new(payer, &mint)?.public;
    send(
        rpc_client,
        &format!("Publishing pubkey {}", elgamal_pk),
        &[
            stealth::instruction::publish_elgamal_pubkey(
                &payer.pubkey(),
                &mint,
                elgamal_pk.into(),
            ),
        ],
        &[payer],
    )?;

    Ok(())
}

fn process_close_elgamal_pubkey(
    rpc_client: &RpcClient,
    payer: &dyn Signer,
    _config: &Config,
    params: &ElGamalPubkeyParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let mint = Pubkey::new(
        bs58::decode(&params.mint).into_vec()?.as_slice()
    );

    send(
        rpc_client,
        &format!("Closing pubkey"),
        &[
            stealth::instruction::close_elgamal_pubkey(
                &payer.pubkey(),
                &mint,
            ),
        ],
        &[payer],
    )?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instruction_buffer_param =
        Arg::with_name("instruction_buffer")
            .long("instruction_buffer")
            .value_name("PUBKEY_STRING")
            .takes_value(true)
            .global(true)
            .help("Instruction buffer pubkey to use");
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
            )
        )
        .subcommand(
            SubCommand::with_name("publish_elgamal_pubkey")
            .about("Write the elgamal pubkey associated with KEYPAIR and the mint to chain")
            .arg(
                Arg::with_name("mint")
                    .long("mint")
                    .value_name("PUBKEY_STRING")
                    .takes_value(true)
                    .global(true)
            )
        )
        .subcommand(
            SubCommand::with_name("close_elgamal_pubkey")
            .about("Close the elgamal pubkey buffer associated with KEYPAIR and the mint")
            .arg(
                Arg::with_name("mint")
                    .long("mint")
                    .value_name("PUBKEY_STRING")
                    .takes_value(true)
                    .global(true)
            )
        )
        .subcommand(
            SubCommand::with_name("create_instruction_buffer")
            .about("Initialize a new instruction buffer with the stealth verification DSL")
            .arg(
                Arg::with_name("instruction_buffer")
                    .long("instruction_buffer")
                    .value_name("KEYPAIR_STRING")
                    .takes_value(true)
                    .global(true)
                    .help("Input buffer keypair to use (or create)")
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
                    instruction_buffer: sub_m.value_of("instruction_buffer").unwrap().to_string(),
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
        ("publish_elgamal_pubkey", Some(sub_m)) => {
            process_publish_elgamal_pubkey(
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
        ("close_elgamal_pubkey", Some(sub_m)) => {
            process_close_elgamal_pubkey(
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
        ("create_instruction_buffer", Some(sub_m)) => {
            let instruction_buffer = specified_or_new(sub_m.value_of("instruction_buffer").map(|s| s.into()));
            println!("Instruction buffer keypair: {}", instruction_buffer.to_base58_string());
            ensure_create_instruction_buffer(
                &rpc_client,
                config.default_signer.as_ref(),
                &instruction_buffer,
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
