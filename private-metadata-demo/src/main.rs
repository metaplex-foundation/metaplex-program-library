use {
    clap::{crate_description, crate_name, crate_version, App, Arg},
    solana_clap_utils::{
        input_validators::{is_url_or_moniker, is_valid_signer, normalize_to_url_if_moniker},
        keypair::DefaultSigner,
    },
    solana_client::{client_error, rpc_client::RpcClient},
    solana_remote_wallet::remote_wallet::RemoteWalletManager,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        instruction::Instruction,
        message::Message,
        program_pack::Pack,
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
    dest_keypair: Option<String>,
    transfer_buffer: Option<String>,
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

fn process_demo(
    rpc_client: &RpcClient,
    payer: &dyn Signer,
    dest_keypair: &Option<String>,
    transfer_buffer: &Option<String>,
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


    let keypair_b = if let Some(kp) = dest_keypair {
        Keypair::from_base58_string(kp)
    } else {
        Keypair::new()
    };

    let elgamal_keypair_b = ElGamalKeypair::new(&keypair_b, &nft_mint)?;
    let elgamal_pk_b = elgamal_keypair_b.public;

    println!("Dest keypair: {}", keypair_b.to_base58_string());
    println!("Dest elgamal pubkey: {}", elgamal_pk_b);

    let private_metadata_key = private_metadata::instruction::get_private_metadata_address(&nft_mint).0;

    if rpc_client.get_account_data(&private_metadata_key)?.len() == 0 {
        let configure_metadata_ix = private_metadata::instruction::configure_metadata(
            payer.pubkey(),
            nft_mint,
            private_metadata::instruction::ConfigureMetadataData {
                elgamal_pk: elgamal_pk_a.into(),
                encrypted_cipher_key: [
                    elgamal_pk_a.encrypt(0 as u32).into(),
                    elgamal_pk_a.encrypt(1 as u32).into(),
                    elgamal_pk_a.encrypt(2 as u32).into(),
                    elgamal_pk_a.encrypt(3 as u32).into(),
                    elgamal_pk_a.encrypt(4 as u32).into(),
                    elgamal_pk_a.encrypt(5 as u32).into(),
                ],
                uri: private_metadata::state::URI([0; 100]),
            },
        );

        send(
            rpc_client,
            &format!("Configuring private metadata: {}", private_metadata_key),
            &[
                configure_metadata_ix,
            ],
            &[payer],
        )?;
    }

    let transfer_buffer = if let Some(kp) = transfer_buffer {
        Keypair::from_base58_string(kp)
    } else {
        Keypair::new()
    };

    if rpc_client.get_account_data(&transfer_buffer.pubkey())?.len() == 0 {
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
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .arg({
            let arg = Arg::with_name("config_file")
                .short("C")
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
                .short("v")
                .takes_value(false)
                .global(true)
                .help("Show additional information"),
        )
        .arg(
            Arg::with_name("json_rpc_url")
                .short("u")
                .long("url")
                .value_name("URL")
                .takes_value(true)
                .global(true)
                .validator(is_url_or_moniker)
                .help("JSON RPC URL for the cluster [default: value from configuration file]"),
        )
        .arg(
            Arg::with_name("dest_keypair")
                .long("dest_keypair")
                .value_name("DEST_KEYPAIR")
                .takes_value(true)
                .global(true)
                .help("Destination keypair to encrypt to"),
        )
        .arg(
            Arg::with_name("transfer_buffer")
                .long("transfer_buffer")
                .value_name("TRANSFER_BUFFER")
                .takes_value(true)
                .global(true)
                .help("Transfer buffer keypair to use (or create)"),
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
            dest_keypair: matches.value_of("dest_keypair").map(|s| s.into()),
            transfer_buffer: matches.value_of("transfer_buffer").map(|s| s.into()),
        }
    };
    solana_logger::setup_with_default("solana=info");

    if config.verbose {
        println!("JSON RPC URL: {}", config.json_rpc_url);
    }
    let rpc_client =
        RpcClient::new_with_commitment(config.json_rpc_url.clone(), config.commitment_config);

    process_demo(
        &rpc_client,
        config.default_signer.as_ref(),
        &config.dest_keypair,
        &config.transfer_buffer,
    ).unwrap_or_else(|err| {
        eprintln!("error: {}", err);
        exit(1);
    });

    Ok(())
}
