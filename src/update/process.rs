use std::str::FromStr;

use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use console::style;
use mpl_candy_machine_core::{
    accounts as nft_accounts, instruction as nft_instruction, CandyMachineData,
};

use crate::{
    cache::load_cache,
    candy_machine::{get_candy_machine_state, CANDY_MACHINE_ID},
    common::*,
    config::{data::ConfigData, parser::get_config_data},
    utils::{assert_correct_authority, spinner_with_style},
};

pub struct UpdateArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub new_authority: Option<String>,
    pub config: String,
    pub candy_machine: Option<String>,
}

pub fn process_update(args: UpdateArgs) -> Result<()> {
    let sugar_config = sugar_setup(args.keypair, args.rpc_url)?;
    let client = setup_client(&sugar_config)?;
    let config_data = get_config_data(&args.config)?;

    // the candy machine id specified takes precedence over the one from the cache
    let candy_machine_id = match args.candy_machine {
        Some(candy_machine_id) => candy_machine_id,
        None => {
            let cache = load_cache(&args.cache, false)?;
            cache.program.candy_machine
        }
    };

    let candy_pubkey = match Pubkey::from_str(&candy_machine_id) {
        Ok(candy_pubkey) => candy_pubkey,
        Err(_) => {
            let error = anyhow!("Failed to parse candy machine id: {}", candy_machine_id);
            error!("{:?}", error);
            return Err(error);
        }
    };

    println!(
        "{} {}Loading candy machine",
        style("[1/2]").bold().dim(),
        LOOKING_GLASS_EMOJI
    );
    println!("{} {}", style("Candy machine ID:").bold(), candy_machine_id);

    let pb = spinner_with_style();
    pb.set_message("Connecting...");

    let candy_machine_state = get_candy_machine_state(&sugar_config, &candy_pubkey)?;
    let candy_machine_data = create_candy_machine_data(&config_data, &candy_machine_state.data)?;

    pb.finish_with_message("Done");

    assert_correct_authority(
        &sugar_config.keypair.pubkey(),
        &candy_machine_state.authority,
    )?;

    println!(
        "\n{} {}Updating configuration",
        style("[2/2]").bold().dim(),
        COMPUTER_EMOJI
    );

    let program = client.program(CANDY_MACHINE_ID);
    let builder = program
        .request()
        .accounts(nft_accounts::Update {
            candy_machine: candy_pubkey,
            authority: program.payer(),
        })
        .args(nft_instruction::Update {
            data: candy_machine_data,
        });

    let pb = spinner_with_style();
    pb.set_message("Sending update transaction...");

    let update_signature = builder.send()?;

    pb.finish_with_message(format!(
        "{} {}",
        style("Update signature:").bold(),
        update_signature
    ));

    if let Some(new_authority) = args.new_authority {
        let pb = spinner_with_style();
        pb.set_message("Sending update authority transaction...");

        let new_authority_pubkey = Pubkey::from_str(&new_authority)?;
        let builder = program
            .request()
            .accounts(nft_accounts::SetAuthority {
                candy_machine: candy_pubkey,
                authority: program.payer(),
            })
            .args(nft_instruction::SetAuthority {
                new_authority: new_authority_pubkey,
            });

        let authority_signature = builder.send()?;
        pb.finish_with_message(format!(
            "{} {}",
            style("Authority signature:").bold(),
            authority_signature
        ));
    }

    Ok(())
}

fn create_candy_machine_data(
    config: &ConfigData,
    candy_machine: &CandyMachineData,
) -> Result<CandyMachineData> {
    let hidden_settings = config.hidden_settings.as_ref().map(|s| s.to_candy_format());

    let creators = config
        .creators
        .clone()
        .into_iter()
        .map(|c| c.to_candy_format())
        .collect::<Result<Vec<mpl_candy_machine_core::Creator>>>()?;

    let data = CandyMachineData {
        symbol: config.symbol.clone(),
        seller_fee_basis_points: config.royalties,
        max_supply: 0,
        is_mutable: config.is_mutable,
        creators,
        hidden_settings,
        config_line_settings: candy_machine.config_line_settings.clone(),
        items_available: config.size,
    };
    Ok(data)
}
