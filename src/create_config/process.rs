use std::{
    default::Default,
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use anchor_lang::prelude::Pubkey;
use anyhow::{anyhow, Result};
use chrono::DateTime;
use console::style;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use url::Url;

use crate::{
    candy_machine::CANDY_MACHINE_ID,
    config::{
        parse_string_as_date, ConfigData, Creator, EndSettingType, EndSettings, GatekeeperConfig,
        HiddenSettings, UploadMethod, WhitelistMintMode, WhitelistMintSettings,
    },
    constants::*,
    setup::{setup_client, sugar_setup},
    upload::list_files,
    utils::{check_spl_token, check_spl_token_account, get_dialoguer_theme},
    validate::Metadata,
};

/// Default name of the first metadata file.
const DEFAULT_METADATA: &str = "0.json";

/// Default value to represent an invalid seller fee basis points.
const INVALID_SELLER_FEE: u16 = u16::MAX;

/// Date mask for formatting input.
const DATE_MASK: &str = "%Y-%m-%d %H:%M:%S %z";

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

    let float_validator = |input: &String| -> Result<(), String> {
        if !input.is_empty() && input.parse::<f64>().is_err() {
            Err(format!(
                "Couldn't parse price input of '{}' to a float.",
                input
            ))
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

    let date_validator = |input: &String| -> Result<(), String> {
        if parse_string_as_date(input).is_err() {
            Err(format!(
                "Couldn't parse input of '{}' to a valid date.",
                input
            ))
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

    let mut symbol: String = Default::default();
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

        symbol = metadata.symbol;

        // Optional in the JSON, so if it doesn't exist, we'll use the default value INVALID value.
        if let Some(sfbp) = metadata.seller_fee_basis_points {
            seller_fee = sfbp;
        }
    }

    println!("\nCheck out our Candy Machine config docs to learn about the options:");
    println!(
        "  -> {}\n",
        style("https://docs.metaplex.com/tools/sugar/configuration")
            .bold()
            .magenta()
            .underlined()
    );

    // price

    config_data.price = Input::with_theme(&theme)
        .with_prompt("What is the price of each NFT?")
        .validate_with(float_validator)
        .interact()
        .unwrap()
        .parse::<f64>()
        .expect("Failed to parse string into u64 that should have already been validated.");

    // number

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
            .with_prompt(
                "What is the symbol of your collection? This must match what is in your asset files. Hit [ENTER] for no symbol.",
            )
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

    // date

    let null_or_none = |input: &str| -> bool { input == "none" || input == "null" };
    let date= Input::with_theme(&theme)
    .with_prompt("What is your go live date? Enter it this format, YYYY-MM-DD HH:MM:SS [+/-]UTC-OFFSET or type 'now' for \
     current time. For example 2022-05-02 18:00:00 +0000 for May 2, 2022 18:00:00 UTC.")
     .validate_with(|input: &String| {
        let trimmed = input.trim().to_lowercase();
        if trimmed == "now" || null_or_none(&trimmed) || parse_string_as_date(input).is_ok() {
            Ok(())
        } else {
            Err("Invalid date format. Format must be YYYY-MM-DD HH:MM:SS [+/-]UTC-OFFSET or 'now'.")
        }
    })
    .interact()
    .unwrap();

    let trimmed = date.trim().to_lowercase();

    config_data.go_live_date = if trimmed == "now" {
        let current_time = chrono::Utc::now();
        Some(current_time.format("%d %b %Y %H:%M:%S %z").to_string())
    } else if null_or_none(&trimmed) {
        None
    } else {
        let date = DateTime::parse_from_str(&date, DATE_MASK)?;
        Some(date.format("%d %b %Y %H:%M:%S %z").to_string())
    };

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

    const SPL_INDEX: usize = 0;
    const GATEKEEPER_INDEX: usize = 1;
    const WL_INDEX: usize = 2;
    const END_SETTINGS_INDEX: usize = 3;
    const HIDDEN_SETTINGS_INDEX: usize = 4;

    let extra_functions_options = vec![
        "SPL Token Mint",
        "Gatekeeper",
        "Whitelist Mint",
        "End Settings",
        "Hidden Settings",
    ];

    let choices = MultiSelect::with_theme(&theme)
        .with_prompt("Which extra features do you want to use? (use [SPACEBAR] to select options you want and hit [ENTER] when done)")
        .items(&extra_functions_options)
        .interact()?;

    // SPL token mint

    let sugar_config = sugar_setup(args.keypair, args.rpc_url)?;
    let client = Arc::new(setup_client(&sugar_config)?);
    let program = client.program(CANDY_MACHINE_ID);

    if choices.contains(&SPL_INDEX) {
        config_data.sol_treasury_account = None;
        config_data.spl_token = Some(
            Pubkey::from_str(
                &Input::with_theme(&theme)
                    .with_prompt("What is your SPL token mint address?")
                    .validate_with(pubkey_validator)
                    .validate_with(|input: &String| -> Result<()> {
                        check_spl_token(&program, input)?;
                        Ok(())
                    })
                    .interact()
                    .unwrap(),
            )
            .expect("Failed to parse string into pubkey that should have already been validated."),
        );
        config_data.spl_token_account = Some(
                Pubkey::from_str(
                    &Input::with_theme(&theme)
                        .with_prompt("What is your SPL token account address (the account that will hold the SPL token mints)?")
                        .validate_with(pubkey_validator)
                        .validate_with(|input: &String| -> Result<()> {
                            check_spl_token_account(&program, input)
                        })
                        .interact()
                        .unwrap(),
                )
                    .expect("Failed to parse string into pubkey that should have already been validated."),
            )
    } else {
        config_data.spl_token = None;
        config_data.spl_token_account = None;
        config_data.sol_treasury_account = Some(
            Pubkey::from_str(
                &Input::with_theme(&theme)
                    .with_prompt("What is your SOL treasury address?")
                    .validate_with(pubkey_validator)
                    .interact()
                    .unwrap(),
            )
            .expect("Failed to parse string into pubkey that should have already been validated."),
        );
    };

    // gatekeeper

    config_data.gatekeeper = if choices.contains(&GATEKEEPER_INDEX) {
        let gatekeeper_options = vec!["Civic Pass", "Verify by Encore"];
        let civic_network = Pubkey::from_str(CIVIC_NETWORK).unwrap();
        let encore_network = Pubkey::from_str(ENCORE_NETWORK).unwrap();
        let selection = Select::with_theme(&theme)
            .with_prompt("Which gatekeeper network do you want to use? Check https://docs.metaplex.com/guides/archived/candy-machine-v2/configuration#provider-networks for more info.")
            .items(&gatekeeper_options)
            .default(0)
            .interact()?;
        let gatekeeper_network = match selection {
            0 => civic_network,
            1 => encore_network,
            _ => civic_network,
        };

        let expire_on_use = Confirm::with_theme(&theme)
            .with_prompt("To help prevent bots even more, do you want to expire the gatekeeper token on each mint?").interact()?;
        Some(GatekeeperConfig::new(gatekeeper_network, expire_on_use))
    } else {
        None
    };

    // whitelist mint settings

    config_data.whitelist_mint_settings = if choices.contains(&WL_INDEX) {
        let mint = Pubkey::from_str(
            &Input::with_theme(&theme)
                .with_prompt("What is your WL token mint address?")
                .validate_with(pubkey_validator)
                .interact()
                .unwrap(),
        )
        .expect("Failed to parse string into pubkey that should have already been validated.");

        let whitelist_mint_mode: WhitelistMintMode = if Confirm::with_theme(&theme)
            .with_prompt("Do you want the whitelist token to be burned on each mint?")
            .interact()?
        {
            WhitelistMintMode::BurnEveryTime
        } else {
            WhitelistMintMode::NeverBurn
        };

        let presale = Confirm::with_theme(&theme)
            .with_prompt("Do you want to enable presale mint with your whitelist token?")
            .interact()?;
        let discount_price: Option<f64> = if presale {
            let price = Input::with_theme(&theme)
                    .with_prompt(
                        "What is the discount price for the presale? Hit [ENTER] to not set a discount price.",
                    )
                    .allow_empty(true)
                    .validate_with(float_validator)
                    .interact()
                    .unwrap();
            if price.is_empty() {
                // the discount price can be set to null
                None
            } else {
                Some(price.parse::<f64>().expect(
                    "Failed to parse string into f64 that should have already been validated.",
                ))
            }
        } else {
            None
        };
        Some(WhitelistMintSettings::new(
            whitelist_mint_mode,
            mint,
            presale,
            discount_price,
        ))
    } else {
        None
    };

    // end settings

    config_data.end_settings = if choices.contains(&END_SETTINGS_INDEX) {
        let end_settings_options = vec!["Date", "Amount"];
        let end_setting_type = match Select::with_theme(&theme)
            .with_prompt("What end settings type do you want to use?")
            .items(&end_settings_options)
            .default(0)
            .interact()
            .unwrap()
        {
            0 => EndSettingType::Date,
            1 => EndSettingType::Amount,
            _ => EndSettingType::Date,
        };

        let number = match end_setting_type {
            EndSettingType::Amount => Input::with_theme(&theme)
                .with_prompt("What is the amount to stop the mint?")
                .validate_with(number_validator)
                .validate_with(|num: &String| {
                    if num.parse::<u64>().unwrap() < config_data.number {
                        Ok(())
                    } else {
                        Err("Your end settings amount cannot be more than the number of items in your candy machine.")
                    }
                })
                .interact()
                .unwrap()
                .parse::<u64>()
                .expect("Failed to parse number into u64 that should have already been validated."),
            EndSettingType::Date => {
                let date = Input::with_theme(&theme)
                    .with_prompt("What is the date to stop the mint? Enter it this format, YYYY-MM-DD HH:MM:SS [+/-]UTC-OFFSET. \
                    For example 2022-05-02 18:00:00 +0000 for May 2, 2022 18:00:00 UTC.")
                    .validate_with(date_validator)
                    .interact()
                    .unwrap();
                    let date_time = DateTime::parse_from_str(&date, DATE_MASK)?;
                    date_time.timestamp() as u64
            }
        };

        Some(EndSettings::new(end_setting_type, number))
    } else {
        None
    };

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
        let hash = Input::with_theme(&theme)
            .with_prompt("What is the hash value for your hidden settings?")
            .validate_with(|hash: &String| {
                if hash.len() != 32 {
                    Err("Your hidden settings hash has to be 32 characters long.")
                } else {
                    Ok(())
                }
            })
            .interact()
            .unwrap();
        Some(HiddenSettings::new(name, uri, hash))
    } else {
        None
    };

    // upload method

    let upload_options = vec!["Bundlr", "AWS", "NFT Storage", "SHDW"];
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
        _ => UploadMethod::Bundlr,
    };

    if config_data.upload_method == UploadMethod::AWS {
        config_data.aws_s3_bucket = Some(
            Input::with_theme(&theme)
                .with_prompt("What is the AWS S3 bucket name?")
                .interact()
                .unwrap(),
        );
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

    // retain authority

    config_data.retain_authority = Confirm::with_theme(&theme)
        .with_prompt("Do you want to retain update authority on your NFTs? We HIGHLY recommend you choose yes.")
        .interact()?;

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
