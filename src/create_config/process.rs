use anchor_lang::prelude::Pubkey;
use anyhow::Result;
use console::{style, Style};
use dialoguer::Confirm;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use std::fs::OpenOptions;
use std::path::Path;
use std::str::FromStr;
use url::Url;

use crate::common::{CANDY_EMOJI, CONFETTI_EMOJI};
use crate::config::{
    go_live_date_as_timestamp, ConfigData, Creator, EndSettingType, EndSettings, GatekeeperConfig,
    HiddenSettings, UploadMethod, WhitelistMintMode, WhitelistMintSettings,
};
use crate::constants::{DEFAULT_ASSETS, DEFAULT_CONFIG};
use crate::upload::count_files;

pub struct CreateConfigArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub config: Option<String>,
}

pub fn process_create_config(args: CreateConfigArgs) -> Result<()> {
    let mut config_data: ConfigData = ConfigData::default();
    let theme = ColorfulTheme {
        prompt_style: Style::new(),
        ..Default::default()
    };

    let pubkey_validator = |input: &String| -> Result<(), String> {
        if Pubkey::from_str(input).is_err() {
            Err(format!("Couldn't parse input of '{}' to a pubkey!", input))
        } else {
            Ok(())
        }
    };

    let float_validator = |input: &String| -> Result<(), String> {
        if input.parse::<f64>().is_err() {
            Err(format!(
                "Couldn't parse price input of '{}' to a float!",
                input
            ))
        } else {
            Ok(())
        }
    };
    let number_validator = |input: &String| -> Result<(), String> {
        if input.parse::<u64>().is_err() || input.parse::<u8>().is_err() {
            Err(format!("Couldn't parse input of '{}' to a number!", input))
        } else {
            Ok(())
        }
    };

    let date_validator = |input: &String| -> Result<(), String> {
        if go_live_date_as_timestamp(input).is_err() {
            Err(format!("Couldn't parse input of '{}' to a date!", input))
        } else {
            Ok(())
        }
    };
    let url_validator = |input: &String| -> Result<(), String> {
        if Url::parse(input).is_err() {
            Err(format!(
                "Couldn't parse input of '{}' to a valid uri!",
                input
            ))
        } else {
            Ok(())
        }
    };

    println!(
        "{}{} {}",
        CANDY_EMOJI,
        style("Sugar Interactive Config Maker")
            .bold()
            .cyan()
            .underlined(),
        CANDY_EMOJI
    );
    println!(
        "{}{}{}\n",
        style("Check out our Candy Machine config docs at ").magenta(),
        style("https://docs.metaplex.com/candy-machine-v2/configuration")
            .bold()
            .underlined()
            .magenta(),
        style(" to learn about the options!").magenta()
    );

    config_data.price = Input::with_theme(&theme)
        .with_prompt("What is the price of each NFT?")
        .validate_with(float_validator)
        .interact()
        .unwrap()
        .parse::<f64>()
        .expect("Failed to parse string into u64 that should have already been validated.");

    let num_files = count_files(DEFAULT_ASSETS);
    let num_files_ok = num_files.as_ref().map(|num| num % 2 == 0).unwrap_or(false);
    config_data.number = if num_files_ok && Confirm::with_theme(&theme)
        .with_prompt(
            format!(
                "I found {} file pairs in the default assets directory. Is this how many NFTs you will have in your candy machine?", num_files.as_ref().unwrap() / 2
            )
        )
        .interact()? {
        (num_files.unwrap() / 2) as u64
    } else {
        Input::with_theme(&theme)
            .with_prompt("How many NFTs will you have in your candy machine?")
            .validate_with(number_validator)
            .interact()
            .unwrap().parse::<u64>().expect("Failed to parse number into u64 that should have already been validated.")
    };

    config_data.go_live_date = Input::with_theme(&theme)
        .with_prompt("What is your go live date? Enter it in RFC 3339 format, i.e., \"2022-02-25T13:00:00Z\", which is 1:00 PM UTC on Feburary 25, 2022.")
        .validate_with(date_validator)
        .interact()
        .unwrap();

    let num_creators = Input::with_theme(&theme)
        .with_prompt("How many creators do you have? (Max limit of 4)")
        .validate_with(number_validator)
        .validate_with({
            |input: &String| match input.parse::<u8>().unwrap() {
                1 | 2 | 3 | 4 => Ok(()),
                _ => Err("Number of creators must be between 1 and 4, inclusive!"),
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
                .with_prompt(format!("Enter creator address #{}", i + 1))
                .validate_with(pubkey_validator)
                .interact()
                .unwrap(),
        )
            .expect("Failed to parse string into pubkey that should have already been validated.");

        let share = Input::with_theme(&theme)
            .with_prompt(format!(
                "Enter royalty percentage share for creator #{} (i.e. 70). Total shares must add to 100!",
                i + 1
            ))
            .validate_with(number_validator)
            .validate_with({
                |input: &String| -> Result<(), &str> {
                    if input.parse::<u8>().unwrap() + total_share > 100 {
                        Err("Royalty share total has exceeded 100 percent!")
                    } else if i == num_creators && input.parse::<u8>().unwrap() + total_share != 100 {
                        Err("Royalty share for all creators must total 100 percent!")
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

    const GATEKEEPER_INDEX: usize = 0;
    const SPL_INDEX: usize = 1;
    const WL_INDEX: usize = 2;
    const END_SETTINGS_INDEX: usize = 3;
    const HIDDEN_SETTINGS_INDEX: usize = 4;
    let extra_functions_options = vec![
        "Gatekeeper",
        "SPL Token Mint",
        "Whitelist Mint",
        "End Settings",
        "Hidden Settings",
    ];

    let choices = MultiSelect::with_theme(&theme)
        .with_prompt(
            "Which extra features do you want to use? (Use spacebar to select options you want. Hit enter for when done.)",
        )
        .items(&extra_functions_options)
        .interact()?;

    config_data.gatekeeper = if choices.contains(&GATEKEEPER_INDEX) {
        let gatekeeper_options = vec!["Civic Pass", "Verify by Encore"];
        let civic_network =
            Pubkey::from_str("ignREusXmGrscGNUesoU9mxfds9AiYTezUKex2PsZV6").unwrap();
        let encore_network =
            Pubkey::from_str("tibePmPaoTgrs929rWpu755EXaxC7M3SthVCf6GzjZt").unwrap();
        let selection = Select::with_theme(&theme)
            .with_prompt("Which gatekeeper do you want to use? Check https://docs.metaplex.com/candy-machine-v2/configuration#provider-networks for more info.")
            .items(&gatekeeper_options)
            .default(0)
            .interact()?;
        let gatekeeper_network = match selection {
            0 => civic_network,
            1 => encore_network,
            _ => civic_network,
        };

        let expire_on_use = Confirm::with_theme(&theme)
            .with_prompt("To help prevent bots even more, do you want to expire the gateway token on each mint?").interact()?;
        Some(GatekeeperConfig::new(gatekeeper_network, expire_on_use))
    } else {
        None
    };

    if choices.contains(&SPL_INDEX) {
        config_data.sol_treasury_account = None;
        config_data.spl_token = Some(
            Pubkey::from_str(
                &Input::with_theme(&theme)
                    .with_prompt("What is your SPL token mint?")
                    .validate_with(pubkey_validator)
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
            .with_prompt("Do you want the whitelist token to be burned each time someone mints?")
            .interact()?
        {
            WhitelistMintMode::BurnEveryTime
        } else {
            WhitelistMintMode::NeverBurn
        };

        let presale = Confirm::with_theme(&theme)
            .with_prompt("Do you want to have a presale mint with your whitelist token?")
            .interact()?;
        let discount_price: Option<f64> = if presale {
            Some(
                Input::with_theme(&theme)
                    .with_prompt("What is the discount price for the presale?")
                    .validate_with(float_validator)
                    .interact()
                    .unwrap()
                    .parse::<f64>()
                    .expect(
                        "Failed to parse string into f64 that should have already been validated.",
                    ),
            )
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
                .with_prompt("What is your end settings ammount?")
                .validate_with(number_validator)
                .validate_with(|num: &String| {
                    if num.parse::<u64>().unwrap() < config_data.number {
                        Ok(())
                    } else {
                        Err("Your end settings ammount can't be more than the number of items in your candy machine!")
                    }
                })
                .interact()
                .unwrap()
                .parse::<u64>()
                .expect("Failed to parse number into u64 that should have already been validated."),
            EndSettingType::Date => {
                let date = Input::with_theme(&theme)
                    .with_prompt("What is your end settings date? Enter it in RFC 3339 format, i.e., \"2022-02-25T13:00:00Z\", which is 1:00 PM UTC on Feburary 25, 2022.")
                    .validate_with(date_validator)
                    .interact()
                    .unwrap();
                go_live_date_as_timestamp(&date).expect("Failed to parse string into timestamp that should have already been validated!") as u64
            }
        };

        Some(EndSettings::new(end_setting_type, number))
    } else {
        None
    };

    config_data.hidden_settings = if choices.contains(&HIDDEN_SETTINGS_INDEX) {
        let name = Input::with_theme(&theme)
            .with_prompt("What is your hidden settings name?")
            .validate_with(|name: &String| {
                if name.len() > 27 {
                    Err("Your hidden settings name probably can't be longer than 27 characters!")
                } else {
                    Ok(())
                }
            })
            .interact()
            .unwrap();
        let uri = Input::with_theme(&theme)
            .with_prompt("What is your hidden settings uri?")
            .validate_with(|uri: &String| {
                if uri.len() > 200 {
                    Err("Your uri can't be longer than 200 characters!")
                } else {
                    Ok(())
                }
            })
            .validate_with(url_validator)
            .interact()
            .unwrap();
        let hash = Input::with_theme(&theme)
            .with_prompt("What is your hidden settings hash?")
            .validate_with(|name: &String| {
                if name.len() != 32 {
                    Err("Your hidden settings hash has to be 32 characters long!")
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

    let upload_options = vec!["Bundlr", "AWS"];
    config_data.upload_method = match Select::with_theme(&theme)
        .with_prompt("What upload method do you want to use?")
        .items(&upload_options)
        .default(0)
        .interact()
        .unwrap()
    {
        0 => UploadMethod::Bundlr,
        1 => UploadMethod::AWS,
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

    config_data.retain_authority = Confirm::with_theme(&theme).with_prompt("Do you want to retain update authority on your NFTs? We HIGHLY reccomend you choose yes!").interact()?;
    config_data.is_mutable = Confirm::with_theme(&theme)
        .with_prompt("Do you want your NFTs to remain mutable? We HIGHLY recommend you choose yes!")
        .interact()?;

    println!();
    let mut save_file = true;
    let file_path = match args.config {
        Some(config) => config,
        None => DEFAULT_CONFIG.to_string(),
    };

    if Path::new(&file_path).is_file() {
        save_file = Select::with_theme(&theme)
            .with_prompt(format!("The file \"{}\" already exists! Do you want to overwrite it with the new config or log the new config to the console?", file_path))
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
                    style(format!("Saving config file: \"{}\"", file_path)).dim()
                );
                serde_json::to_writer_pretty(f, &config_data)
                    .expect("Unable to convert config to JSON!");
                println!(
                    "{}{} {}",
                    CONFETTI_EMOJI,
                    style("Successfully generated the config file!")
                        .bold()
                        .green(),
                    CONFETTI_EMOJI
                )
            }

            Err(_) => {
                println!(
                    "{}",
                    style("Error creating config file! Logging config to console!\n")
                        .bold()
                        .red()
                );
                println!(
                    "{}",
                    style(
                        serde_json::to_string_pretty(&config_data)
                            .expect("Unable to convert config to JSON!")
                    )
                    .red()
                );
            }
        }
    } else {
        println!("{}", style("Logging config to console!\n").dim());
        println!(
            "{}",
            style(
                serde_json::to_string_pretty(&config_data)
                    .expect("Unable to convert config to JSON!")
            )
            .green()
        );
    }

    Ok(())
}
