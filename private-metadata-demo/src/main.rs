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
) -> Result<(), Box<dyn std::error::Error>> {

    let nft_mint = Pubkey::new_from_array([
       94, 126,  13, 209, 184, 218, 233,  14,
       52, 162,  23,  15, 236, 168, 144, 134,
      125, 127,  10,  85, 102,  76, 157,  51,
       54, 190, 208,  98,  39,  30, 196,  87
    ]);

    let elgamal_keypair_a = ElGamalKeypair::new(payer, &nft_mint)?;
    let elgamal_pk_a = elgamal_keypair_a.public;

    println!("Payer elgamal pubkey: {}", elgamal_pk_a);

    let elgamal_keypair_b = ElGamalKeypair::default();
    let elgamal_pk_b = elgamal_keypair_b.public;

    println!("New elgamal pubkey: {}", elgamal_pk_b);

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
        &format!("Configuring private metadata: {}", nft_mint),
        &[
            configure_metadata_ix,
        ],
        &[payer],
    )?;

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

    process_demo(&rpc_client, config.default_signer.as_ref()).unwrap_or_else(|err| {
        eprintln!("error: {}", err);
        exit(1);
    });

    Ok(())
}
