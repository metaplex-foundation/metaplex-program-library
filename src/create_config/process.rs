use std::{
    default::Default,
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    str::FromStr,
};

use anchor_lang::prelude::Pubkey;
use anyhow::{anyhow, Result};
use console::style;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use url::Url;

use crate::{
    config::{AwsConfig, ConfigData, Creator, HiddenSettings, PinataConfig, UploadMethod},
    constants::*,
    upload::list_files,
    utils::get_dialoguer_theme,
    validate::Metadata,
};

/// Default name of the first metadata file.
const DEFAULT_METADATA: &str = "0.json";

/// Default value to represent an invalid seller fee basis points.
const INVALID_SELLER_FEE: u16 = u16::MAX;
const INVALID_SYMBOL: &str = "abcdefghijklmnopqrstuvwxyz";

pub struct CreateConfigArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub config: Option<String>,
    pub assets_dir: String,
}

pub fn process_create_config(args: CreateConfigArgs) -> Result<()> {
    let mut config_data: ConfigData = ConfigData::default();
    let theme = get_dialoguer_theme();

    // validators

    let pubkey_validator = |input: &String| -> Result<(), String> {
        if Pubkey::from_str(input).is_err() {
            Err(format!("Couldn't parse input of '{}' to a pubkey.", input))
        } else {
            Ok(())
        }
    };

    let number_validator = |input: &String| -> Result<(), String> {
        if input.parse::<u64>().is_err() {
            Err(format!("Couldn't parse input of '{}' to a number.", input))
        } else {
            Ok(())
        }
    };

    let url_validator = |input: &String| -> Result<(), String> {
        if Url::parse(input).is_err() {
            Err(format!(
                "Couldn't parse input of '{}' to a valid uri.",
                input
            ))
        } else {
            Ok(())
        }
    };

    let symbol_validator = |input: &String| -> Result<(), String> {
        if input.len() > 10 {
            Err(String::from("Symbol must be 10 characters or less."))
        } else {
            Ok(())
        }
    };

    let seller_fee_basis_points_validator = |input: &String| -> Result<(), String> {
        let value = match input.parse::<u16>() {
            Ok(value) => value,
            Err(_) => return Err(format!("Couldn't parse input of '{}' to a number.", input)),
        };
        if value > 10_000 {
            Err(String::from(
                "Seller fee basis points must be 10,000 or less.",
            ))
        } else {
            Ok(())
        }
    };

    println!(
        "{} {}Sugar interactive config maker",
        style("[1/2]").bold().dim(),
        CANDY_EMOJI
    );

    // checks if we have an assets dir and count the number of files
    // assumes 0 in case of error since assets_dir is optional
    let num_files = match list_files(&args.assets_dir, false) {
        Ok(number) => number.len(),
        _ => 0,
    };

    let mut symbol: String = INVALID_SYMBOL.to_string();
    let mut seller_fee = INVALID_SELLER_FEE;

    if num_files > 0 {
        println!("\nFound metadata file(s) in folder '{}':", args.assets_dir);
        println!("  -> Loading values from file '{}'", DEFAULT_METADATA);

        // loads the default values from the first metadata file
        let metadata_file = PathBuf::from(&args.assets_dir)
            .join(DEFAULT_METADATA)
            .to_str()
            .expect("Failed to convert metadata path from unicode.")
            .to_string();

        let m = File::open(&metadata_file)?;
        let metadata: Metadata = serde_json::from_reader(m).map_err(|e| {
            anyhow!("Failed to read metadata file '{metadata_file}' with error: {e}")
        })?;

        // Optional in the JSON, so if it doesn't exist, we'll use the default value.
        if let Some(s) = metadata.symbol {
            symbol = s;
        }

        // Optional in the JSON, so if it doesn't exist, we'll use the default value.
        if let Some(sfbp) = metadata.seller_fee_basis_points {
            seller_fee = sfbp;
        }
    }

    println!("\nCheck out our Candy Machine config docs to learn about the options:");
    println!(
        "  -> {}\n",
        style("https://docs.metaplex.com/developer-tools/sugar/guides/configuration")
            .bold()
            .magenta()
            .underlined()
    );

    // size

    config_data.number = if num_files > 0 && (num_files % 2) == 0 && Confirm::with_theme(&theme)
        .with_prompt(
            format!(
                "Found {} file pairs in \"{}\". Is this how many NFTs you will have in your candy machine?", num_files / 2, args.assets_dir,
            )
        )
        .interact()? {
        (num_files / 2) as u64
    } else {
        Input::with_theme(&theme)
            .with_prompt("How many NFTs will you have in your candy machine?")
            .validate_with(number_validator)
            .interact()
            .unwrap().parse::<u64>().expect("Failed to parse number into u64 that should have already been validated.")
    };

    // symbol

    config_data.symbol = if num_files > 0
        && symbol != *INVALID_SYMBOL
        && Confirm::with_theme(&theme)
            .with_prompt(format!(
                "Found {} in your metadata file. Is this value correct?",
                if symbol.is_empty() {
                    "no symbol".to_string()
                } else {
                    format!("symbol \"{}\"", symbol)
                },
            ))
            .interact()?
    {
        symbol
    } else {
        Input::with_theme(&theme)
            .with_prompt("What is the symbol of your collection? Hit [ENTER] for no symbol.")
            .allow_empty(true)
            .validate_with(symbol_validator)
            .interact()
            .unwrap()
    };

    // seller_fee_basis_points

    config_data.seller_fee_basis_points = if num_files > 0 && seller_fee != INVALID_SELLER_FEE && Confirm::with_theme(&theme)
        .with_prompt(
            format!(
                "Found value {} for seller fee basis points in your metadata file. Is this value correct?", seller_fee,
            )
        )
        .interact()? {
        seller_fee
    } else {
        Input::with_theme(&theme)
            .with_prompt(
                "What is the seller fee basis points?",
            )
            .validate_with(seller_fee_basis_points_validator)
            .interact()
            .unwrap()
            .parse::<u16>()
            .expect("Failed to parse number into u16 that should have already been validated.")
    };

    // is sequential

    config_data.is_sequential = Confirm::with_theme(&theme)
        .with_prompt(
            "Do you want to use a sequential mint index generation? We recommend you choose no.",
        )
        .interact()?;

    // creators

    let num_creators = Input::with_theme(&theme)
        .with_prompt("How many creator wallets do you have? (max limit of 4)")
        .validate_with(number_validator)
        .validate_with({
            |input: &String| match input.parse::<u8>().unwrap() {
                1 | 2 | 3 | 4 => Ok(()),
                _ => Err("Number of creator wallets must be between 1 and 4, inclusive."),
            }
        })
        .interact()
        .unwrap()
        .parse::<u8>()
        .expect("Failed to parse number into u8 that should have already been validated.");

    let mut total_share = 0;

    (0..num_creators).into_iter().for_each(|i| {
        let address = Pubkey::from_str(
            &Input::with_theme(&theme)
                .with_prompt(format!("Enter creator wallet address #{}", i + 1))
                .validate_with(pubkey_validator)
                .interact()
                .unwrap(),
        )
            .expect("Failed to parse string into pubkey that should have already been validated.");

        let share = Input::with_theme(&theme)
            .with_prompt(format!(
                "Enter royalty percentage share for creator #{} (e.g., 70). Total shares must add to 100.",
                i + 1
            ))
            .validate_with(number_validator)
            .validate_with({
                |input: &String| -> Result<(), &str> {
                    if input.parse::<u8>().unwrap() + total_share > 100 {
                        Err("Royalty share total has exceeded 100 percent.")
                    } else if i == num_creators && input.parse::<u8>().unwrap() + total_share != 100 {
                        Err("Royalty share for all creators must total 100 percent.")
                    } else {
                        Ok(())
                    }
                }
            })
            .interact()
            .unwrap()
            .parse::<u8>()
            .expect("Failed to parse number into u64 that should have already been validated.");

        total_share += share;
        let creator = Creator { address, share };
        config_data.creators.push(creator);
    });

    const HIDDEN_SETTINGS_INDEX: usize = 0;

    let extra_functions_options = vec!["Hidden Settings"];

    let choices = MultiSelect::with_theme(&theme)
        .with_prompt("Which extra features do you want to use? (use [SPACEBAR] to select options you want and hit [ENTER] when done)")
        .items(&extra_functions_options)
        .interact()?;

    // hidden settings

    config_data.hidden_settings = if choices.contains(&HIDDEN_SETTINGS_INDEX) {
        let name = Input::with_theme(&theme)
            .with_prompt("What is the prefix name for your hidden settings mints? The mint index will be appended at the end of the name.")
            .validate_with(|name: &String| {
                if name.len() > (MAX_NAME_LENGTH - 7) {
                    Err("Your hidden settings name probably cannot be longer than 25 characters.")
                } else {
                    Ok(())
                }
            })
            .interact()
            .unwrap();
        let uri = Input::with_theme(&theme)
            .with_prompt("What is URI to be used for each mint?")
            .validate_with(|uri: &String| {
                if uri.len() > MAX_URI_LENGTH {
                    Err("The URI cannot be longer than 200 characters.")
                } else {
                    Ok(())
                }
            })
            .validate_with(url_validator)
            .interact()
            .unwrap();
        Some(HiddenSettings::new(name, uri, String::from("")))
    } else {
        None
    };

    // upload method
    let upload_options = vec!["Bundlr", "AWS", "NFT Storage", "SHDW", "Pinata"];
    config_data.upload_method = match Select::with_theme(&theme)
        .with_prompt("What upload method do you want to use?")
        .items(&upload_options)
        .default(0)
        .interact()
        .unwrap()
    {
        0 => UploadMethod::Bundlr,
        1 => UploadMethod::AWS,
        2 => UploadMethod::NftStorage,
        3 => UploadMethod::SHDW,
        4 => UploadMethod::Pinata,
        _ => UploadMethod::Bundlr,
    };

    if config_data.upload_method == UploadMethod::AWS {
        let bucket: String = Input::with_theme(&theme)
            .with_prompt("What is the AWS S3 bucket name?")
            .interact()
            .unwrap();

        let profile = Input::with_theme(&theme)
            .with_prompt("What is the AWS profile name?")
            .default(String::from("default"))
            .interact()
            .unwrap();

        let directory = Input::with_theme(&theme)
            .with_prompt("What is the directory to upload to? Leave blank to store files at the bucket root dir.")
            .allow_empty(true)
            .interact()
            .unwrap();

        let domain: String = Input::with_theme(&theme)
            .with_prompt("Do you have a custom domain? Leave blank to use AWS default domain.")
            .allow_empty(true)
            .interact()
            .unwrap();

        config_data.aws_config = Some(AwsConfig::new(
            bucket,
            profile,
            directory,
            if domain.is_empty() {
                None
            } else {
                Some(domain)
            },
        ));
    }

    if config_data.upload_method == UploadMethod::NftStorage {
        config_data.nft_storage_auth_token = Some(
            Input::with_theme(&theme)
                .with_prompt("What is the NFT Storage authentication token?")
                .interact()
                .unwrap(),
        );
    }

    if config_data.upload_method == UploadMethod::SHDW {
        config_data.shdw_storage_account = Some(
            Input::with_theme(&theme)
                .with_prompt("What is the SHDW storage address?")
                .validate_with(pubkey_validator)
                .interact()
                .unwrap(),
        );
    }

    if config_data.upload_method == UploadMethod::Pinata {
        let jwt: String = Input::with_theme(&theme)
            .with_prompt("What is your Pinata JWT authentication?")
            .interact()
            .unwrap();

        let api_gateway = Input::with_theme(&theme)
            .with_prompt("What is the Pinata API gateway for upload?")
            .default(String::from("https://api.pinata.cloud"))
            .interact()
            .unwrap();

        let content_gateway = Input::with_theme(&theme)
            .with_prompt("What is the Pinata gateway for content retrieval?")
            .default(String::from("https://gateway.pinata.cloud"))
            .interact()
            .unwrap();

        let parallel_limit = Input::with_theme(&theme)
            .with_prompt("How many concurrent uploads are allowed?")
            .validate_with(number_validator)
            .interact()
            .unwrap()
            .parse::<u16>()
            .expect("Failed to parse number into u64 that should have already been validated.");

        config_data.pinata_config = Some(PinataConfig {
            jwt,
            api_gateway,
            content_gateway,
            parallel_limit: Some(parallel_limit),
        });
    }

    // is mutable

    config_data.is_mutable = Confirm::with_theme(&theme)
        .with_prompt("Do you want your NFTs to remain mutable? We HIGHLY recommend you choose yes.")
        .interact()?;

    // saving configuration file

    println!(
        "\n{} {}Saving config file\n",
        style("[2/2]").bold().dim(),
        PAPER_EMOJI
    );

    let mut save_file = true;
    let file_path = match args.config {
        Some(config) => config,
        None => DEFAULT_CONFIG.to_string(),
    };

    if Path::new(&file_path).is_file() {
        save_file = Select::with_theme(&theme)
            .with_prompt(format!("The file \"{}\" already exists. Do you want to overwrite it with the new config or log the new config to the console?", file_path))
            .items(&["Overwrite the file", "Log to console"])
            .default(0)
            .interact()
            .unwrap() == 0;
        println!();
    }

    if save_file {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(Path::new(&file_path));

        match file {
            Ok(f) => {
                println!(
                    "{}",
                    style(format!("Saving config to file: \"{}\"\n", file_path))
                );
                serde_json::to_writer_pretty(f, &config_data)
                    .expect("Unable to convert config to JSON!");

                println!(
                    "{} {}",
                    style("Successfully generated the config file.")
                        .magenta()
                        .bold(),
                    CONFETTI_EMOJI
                )
            }

            Err(_) => {
                println!(
                    "{}\n",
                    style("Error creating config file - logging config to console.")
                        .bold()
                        .red()
                );
                println!(
                    "{}",
                    style(
                        serde_json::to_string_pretty(&config_data)
                            .expect("Unable to convert config to JSON.")
                    )
                    .red()
                );
            }
        }
    } else {
        println!("{}\n", style("Logging config to console:").dim());
        println!(
            "{}",
            serde_json::to_string_pretty(&config_data).expect("Unable to convert config to JSON.")
        );
    }

    Ok(())
}
