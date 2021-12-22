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

struct DemoParams {
    dest_keypair: Option<String>,
    transfer_buffer: Option<String>,
    instruction_buffer: Option<String>,
    input_buffer: Option<String>,
    compute_buffer: Option<String>,
}

fn process_demo(
    rpc_client: &RpcClient,
    payer: &dyn Signer,
    _config: &Config,
    DemoParams{
        dest_keypair,
        transfer_buffer,
        instruction_buffer,
        input_buffer,
        compute_buffer,
    }: &DemoParams,
) -> Result<(), Box<dyn std::error::Error>> {

    let nft_mint = Pubkey::new_from_array([
       94, 126,  13, 209, 184, 218, 233,  14,
       52, 162,  23,  15, 236, 168, 144, 134,
      125, 127,  10,  85, 102,  76, 157,  51,
       54, 190, 208,  98,  39,  30, 196,  87
    ]);

    println!("Max metadata: {}", spl_token_metadata::state::MAX_METADATA_LEN);

    println!("NFT mint pubkey: {}", nft_mint);

    let metadata_key = private_metadata::instruction::get_metadata_address(&nft_mint).0;
    let metadata_data = rpc_client.get_account_data(&metadata_key)?;
    println!("NFT metadata pubkey: {}", metadata_key);

    let has_creators_idx = 1 + 32 + 32 + 4 + spl_token_metadata::state::MAX_NAME_LENGTH + 4 + spl_token_metadata::state::MAX_SYMBOL_LENGTH + 4 + spl_token_metadata::state::MAX_URI_LENGTH + 2;
    let has_creators = metadata_data[has_creators_idx];
    println!("Metadata has creators: {}", has_creators);

    let mut is_mutable_idx = has_creators_idx
        + 1 // option
        + 1 // primary sale
        ;

    if has_creators == 1 {
        let num_creators = u32::from_le_bytes(metadata_data[has_creators_idx + 1..has_creators_idx + 5].try_into()?);
        println!("Metadata num creators: {}", num_creators);
        is_mutable_idx += 4 + num_creators as usize * spl_token_metadata::state::MAX_CREATOR_LEN;
    }

    let is_mutable = metadata_data[is_mutable_idx];
    println!("Metadata is mutable {}: {}", is_mutable_idx, is_mutable);

    let elgamal_keypair_a = ElGamalKeypair::new(payer, &nft_mint)?;
    let elgamal_pk_a = elgamal_keypair_a.public;

    println!("Payer elgamal pubkey: {}", elgamal_pk_a);


    let dest_keypair = if let Some(kp) = dest_keypair {
        Keypair::from_base58_string(kp)
    } else {
        Keypair::new()
    };

    let elgamal_keypair_b = ElGamalKeypair::new(&dest_keypair, &nft_mint)?;
    let elgamal_pk_b = elgamal_keypair_b.public;

    println!("Dest keypair: {}", dest_keypair.to_base58_string());
    println!("Dest elgamal pubkey: {}", elgamal_pk_b);

    let private_metadata_key = private_metadata::instruction::get_private_metadata_address(&nft_mint).0;

    let mut private_metadata_data = rpc_client.get_account_data(&private_metadata_key)?;
    if private_metadata_data.len() == 0 {
        let configure_metadata_ix = private_metadata::instruction::configure_metadata(
            payer.pubkey(),
            nft_mint,
            elgamal_pk_a.into(),
            &[
                elgamal_pk_a.encrypt(0 as u32).into(),
                elgamal_pk_a.encrypt(1 as u32).into(),
                elgamal_pk_a.encrypt(2 as u32).into(),
                elgamal_pk_a.encrypt(3 as u32).into(),
                elgamal_pk_a.encrypt(4 as u32).into(),
                elgamal_pk_a.encrypt(5 as u32).into(),
            ],
            &[],
        );

        send(
            rpc_client,
            &format!("Configuring private metadata: {}", private_metadata_key),
            &[
                configure_metadata_ix,
            ],
            &[payer],
        )?;

        private_metadata_data = rpc_client.get_account_data(&private_metadata_key)?;
    } else {
        println!("Private metadata already initialized: {}", private_metadata_key);
    }

    let transfer_buffer = if let Some(kp) = transfer_buffer {
        Keypair::from_base58_string(kp)
    } else {
        Keypair::new()
    };

    let mut transfer_buffer_data = rpc_client.get_account_data(&transfer_buffer.pubkey())?;
    if transfer_buffer_data.len() == 0 {
        let buffer_len = private_metadata::state::CipherKeyTransferBuffer::get_packed_len();
        let buffer_minimum_balance_for_rent_exemption = rpc_client
            .get_minimum_balance_for_rent_exemption(buffer_len)?;

        send(
            rpc_client,
            &format!("Initializing transfer buffer: {}", transfer_buffer.to_base58_string()),
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
                    nft_mint,
                    transfer_buffer.pubkey(),
                    elgamal_pk_b.into(),
                ),
            ],
            &[payer, &transfer_buffer],
        )?;

        transfer_buffer_data = rpc_client.get_account_data(&transfer_buffer.pubkey())?;
    } else {
        println!("Transfer buffer already initialized: {}", transfer_buffer.to_base58_string());
    }


    let transfer_buffer_account = private_metadata::state::CipherKeyTransferBuffer::from_bytes(
        &transfer_buffer_data).ok_or(ProgramError::InvalidArgument)?;

    let private_metadata_account = private_metadata::state::PrivateMetadataAccount::from_bytes(
        &private_metadata_data).ok_or(ProgramError::InvalidArgument)?;


    let input_buffer = if let Some(kp) = input_buffer {
        Keypair::from_base58_string(kp)
    } else {
        Keypair::new()
    };

    let instruction_buffer = if let Some(kp) = instruction_buffer {
        Keypair::from_base58_string(kp)
    } else {
        Keypair::new()
    };

    let compute_buffer = if let Some(kp) = compute_buffer {
        Keypair::from_base58_string(kp)
    } else {
        Keypair::new()
    };

    println!("Instruction buffer keypair: {}", instruction_buffer.to_base58_string());
    println!("Input buffer keypair: {}", input_buffer.to_base58_string());
    println!("Compute buffer keypair: {}", compute_buffer.to_base58_string());

    use curve25519_dalek_onchain::instruction as dalek;

    let dsl = dalek::transer_proof_instructions(vec![3, 3, 5]);
    assert!(
        dsl == private_metadata::equality_proof::DSL_INSTRUCTION_BYTES,
        "DSL does not match!",
    );

    let instruction_buffer_len = (dalek::HEADER_SIZE + dsl.len()) as usize;

    // pick a large number... at least > 8 * 128 * scalars.len()
    let compute_buffer_len = dalek::HEADER_SIZE + 15000;

    let instruction_buffer_data = rpc_client.get_account_data(&instruction_buffer.pubkey());
    if let Ok(data) = instruction_buffer_data {
        assert!(data.len() >= instruction_buffer_len);
    } else {
        send(
            rpc_client,
            &format!("Creating instruction buffer"),
            &[
                system_instruction::create_account(
                    &payer.pubkey(),
                    &instruction_buffer.pubkey(),
                    rpc_client.get_minimum_balance_for_rent_exemption(instruction_buffer_len)?,
                    instruction_buffer_len as u64,
                    &curve25519_dalek_onchain::id(),
                ),
                dalek::initialize_buffer(
                    instruction_buffer.pubkey(),
                    payer.pubkey(),
                    dalek::Key::InstructionBufferV1,
                    vec![],
                ),
            ],
            &[payer, &instruction_buffer],
        )?;

        let mut instructions = vec![];

        // write the instructions
        let mut dsl_idx = 0;
        let dsl_chunk = 800;
        loop {
            let end = (dsl_idx+dsl_chunk).min(dsl.len());
            let done = end == dsl.len();
            instructions.push(
                dalek::write_bytes(
                    instruction_buffer.pubkey(),
                    payer.pubkey(),
                    (dalek::HEADER_SIZE + dsl_idx) as u32,
                    done,
                    &dsl[dsl_idx..end],
                )
            );
            send(
                rpc_client,
                &format!("Writing instructions"),
                instructions.as_slice(),
                &[payer],
            )?;
            instructions.clear();
            if done {
                break;
            } else {
                dsl_idx = end;
            }
        }
    }

    let close_compute_and_input = || {
        let mut instructions = vec![];
        for buffer in [input_buffer.pubkey(), compute_buffer.pubkey()] {
            if let Ok(_) = rpc_client.get_account_data(&input_buffer.pubkey()) {
                instructions.push(
                    dalek::close_buffer(
                        buffer,
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
    };

    // if previous run failed and we respecified...
    close_compute_and_input()?;

    for chunk in 0..private_metadata::state::CIPHER_KEY_CHUNKS {
        let updated_mask = 1<<chunk;
        if transfer_buffer_account.updated & updated_mask == updated_mask {
            continue;
        }

        println!(
            "Existing ct chunk {:#02x?}",
            private_metadata_account.encrypted_cipher_key[chunk],
        );

        let transfer = private_metadata::transfer_proof::TransferData::new(
            &elgamal_keypair_a,
            elgamal_pk_b,
            chunk as u32,
            private_metadata_account.encrypted_cipher_key[chunk].try_into()?,
        );

        let equality_proof = private_metadata::equality_proof::EqualityProof::from_bytes(
            &transfer.proof.equality_proof.0).unwrap();

        let points = [
            // statement inputs
            transfer.transfer_public_keys.src_pubkey.0,
            private_metadata::equality_proof::COMPRESSED_H,
            equality_proof.Y_0.0,

            transfer.transfer_public_keys.dst_pubkey.0,
            transfer.dst_cipher_key_chunk_ct.0[32..].try_into().unwrap(),
            equality_proof.Y_1.0,

            transfer.dst_cipher_key_chunk_ct.0[..32].try_into().unwrap(),
            transfer.src_cipher_key_chunk_ct.0[..32].try_into().unwrap(),
            transfer.src_cipher_key_chunk_ct.0[32..].try_into().unwrap(),
            private_metadata::equality_proof::COMPRESSED_H,
            equality_proof.Y_2.0,
        ];

        use private_metadata::transcript::TranscriptProtocol;
        use private_metadata::transfer_proof::TransferProof;
        use private_metadata::equality_proof::EqualityProof;
        let mut transcript = TransferProof::transcript_new();
        TransferProof::build_transcript(
            &transfer.src_cipher_key_chunk_ct,
            &transfer.dst_cipher_key_chunk_ct,
            &transfer.transfer_public_keys,
            &mut transcript,
        ).unwrap();

        EqualityProof::build_transcript(
            &equality_proof,
            &mut transcript,
        ).unwrap();

        let challenge_c = transcript.challenge_scalar(b"c");

        use curve25519_dalek::scalar::Scalar;
        use curve25519_dalek_onchain::scalar::Scalar as OScalar;
        let scalars = vec![
             equality_proof.sh_1,
             -challenge_c,
             -Scalar::one(),

             equality_proof.rh_2,
             -challenge_c,
             -Scalar::one(),

             challenge_c,
             -challenge_c,
             equality_proof.sh_1,
             -equality_proof.rh_2,
             -Scalar::one(),
        ]
            .iter()
            .map(|s| OScalar::from_canonical_bytes(s.bytes).unwrap())
            .collect::<Vec<_>>();

        let input_buffer_len = dalek::HEADER_SIZE + 11 * 32 * 2 + 128;

        send(
            rpc_client,
            &format!("Creating input and compute buffers"),
            &[
                system_instruction::create_account(
                    &payer.pubkey(),
                    &input_buffer.pubkey(),
                    rpc_client.get_minimum_balance_for_rent_exemption(input_buffer_len)?,
                    input_buffer_len as u64,
                    &curve25519_dalek_onchain::id(),
                ),
                system_instruction::create_account(
                    &payer.pubkey(),
                    &compute_buffer.pubkey(),
                    rpc_client.get_minimum_balance_for_rent_exemption(compute_buffer_len)?,
                    compute_buffer_len as u64,
                    &curve25519_dalek_onchain::id(),
                ),
                dalek::initialize_buffer(
                    input_buffer.pubkey(),
                    payer.pubkey(),
                    dalek::Key::InputBufferV1,
                    vec![],
                ),
                dalek::initialize_buffer(
                    compute_buffer.pubkey(),
                    payer.pubkey(),
                    dalek::Key::ComputeBufferV1,
                    vec![instruction_buffer.pubkey(), input_buffer.pubkey()],
                ),
            ],
            &[payer, &input_buffer, &compute_buffer],
        )?;


        send(
            rpc_client,
            &format!("Writing inputs"),
            dalek::write_input_buffer(
                input_buffer.pubkey(),
                payer.pubkey(),
                &points,
                scalars.as_slice(),
            ).as_slice(),
            &[payer],
        )?;

        let instructions_per_tx = 32;
        let num_cranks = dsl.len() / dalek::INSTRUCTION_SIZE;
        let mut current = 0;
        while current < num_cranks {
            let mut instructions = vec![];
            let iter_start = current;
            for _j in 0..instructions_per_tx {
                if current >= num_cranks {
                    break;
                }
                instructions.push(
                    dalek::crank_compute(
                        instruction_buffer.pubkey(),
                        input_buffer.pubkey(),
                        compute_buffer.pubkey(),
                    ),
                );
                current += 1;
            }
            send(
                rpc_client,
                &format!(
                    "Iterations {}..{}",
                    iter_start,
                    current,
                ),
                instructions.as_slice(),
                &[payer],
            )?;
        }


        let transfer_chunk_ix = private_metadata::instruction::transfer_chunk_slow(
            payer.pubkey(),
            nft_mint,
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

        close_compute_and_input()?;
    }

    Ok(())
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
    params: &DecryptCipherKeyParams ,
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
                    eprintln!("decrypt cipher text error");
                    exit(1);
                }
            }
        })
        .collect::<Vec<_>>();

    let cipher_key_bytes = bytemuck::cast_slice::<u32, u8>(cipher_key_words.as_slice());

    let cipher_key = bs58::encode(cipher_key_bytes).into_string();
    println!("decoded cipher key: {}", cipher_key);

    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            SubCommand::with_name("demo")
            .about("ZK proof transfer demo")
            .arg(
                Arg::with_name("dest_keypair")
                    .long("dest_keypair")
                    .value_name("KEYPAIR_STRING")
                    .takes_value(true)
                    .global(true)
                    .help("Destination keypair to encrypt to"),
            )
            .arg(
                Arg::with_name("transfer_buffer")
                    .long("transfer_buffer")
                    .value_name("KEYPAIR_STRING")
                    .takes_value(true)
                    .global(true)
                    .help("Transfer buffer keypair to use (or create)"),
            )
            .arg(
                Arg::with_name("instruction_buffer")
                    .long("instruction_buffer")
                    .value_name("KEYPAIR_STRING")
                    .takes_value(true)
                    .global(true)
                    .help("Instruction buffer keypair to use (or create)"),
            )
            .arg(
                Arg::with_name("input_buffer")
                    .long("input_buffer")
                    .value_name("KEYPAIR_STRING")
                    .takes_value(true)
                    .global(true)
                    .help("Input buffer keypair to use (or create)"),
            )
            .arg(
                Arg::with_name("compute_buffer")
                    .long("compute_buffer")
                    .value_name("KEYPAIR_STRING")
                    .takes_value(true)
                    .global(true)
                    .help("Compute buffer keypair to use (or create)"),
            )
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
        ("demo", Some(sub_m)) => {
            process_demo(
                &rpc_client,
                config.default_signer.as_ref(),
                &config,
                &DemoParams {
                    dest_keypair: sub_m.value_of("dest_keypair").map(|s| s.into()),
                    transfer_buffer: sub_m.value_of("transfer_buffer").map(|s| s.into()),
                    instruction_buffer: sub_m.value_of("instruction_buffer").map(|s| s.into()),
                    input_buffer: sub_m.value_of("input_buffer").map(|s| s.into()),
                    compute_buffer: sub_m.value_of("compute_buffer").map(|s| s.into()),
                },
            ).unwrap_or_else(|err| {
                eprintln!("error: {}", err);
                exit(1);
            });
        }
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
        _ => {
        }
    }

    Ok(())
}
