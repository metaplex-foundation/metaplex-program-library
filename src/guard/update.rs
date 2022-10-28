use std::str::FromStr;

use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use console::style;
use mpl_candy_guard::{accounts::Update as UpdateAccount, instruction::Update};

use crate::{cache::load_cache, common::*, config::get_config_data, utils::*};

pub struct GuardUpdateArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub config: String,
    pub candy_guard: Option<String>,
}

pub fn process_guard_update(args: GuardUpdateArgs) -> Result<()> {
    println!(
        "{} {}Loading candy guard",
        style("[1/2]").bold().dim(),
        LOOKING_GLASS_EMOJI
    );

    // the candy guard id specified takes precedence over the one from the cache

    let candy_guard_id = if let Some(candy_guard) = args.candy_guard {
        candy_guard
    } else {
        let cache = load_cache(&args.cache, false)?;
        cache.program.candy_guard
    };

    if candy_guard_id.is_empty() {
        return Err(anyhow!("Missing candy guard id."));
    }

    let candy_guard_id = match Pubkey::from_str(&candy_guard_id) {
        Ok(candy_guard_id) => candy_guard_id,
        Err(_) => {
            let error = anyhow!("Failed to parse candy guard id: {}", candy_guard_id);
            error!("{:?}", error);
            return Err(error);
        }
    };

    let sugar_config = sugar_setup(args.keypair, args.rpc_url)?;
    let client = setup_client(&sugar_config)?;
    let program = client.program(mpl_candy_guard::ID);
    let payer = sugar_config.keypair;

    let pb = spinner_with_style();
    pb.set_message("Connecting...");
    // make sure the account exists on-chain
    let _account = program.rpc().get_account(&candy_guard_id)?;
    pb.finish_with_message("Done");

    println!("{} {}", style("Candy guard ID:").bold(), candy_guard_id);

    println!(
        "\n{} {}Updating configuration",
        style("[2/2]").bold().dim(),
        COMPUTER_EMOJI
    );

    let config_data = get_config_data(&args.config)?;
    let data = if let Some(guards) = &config_data.guards {
        guards.to_guard_format()?
    } else {
        return Err(anyhow!("Missing guards configuration."));
    };

    let mut serialized_data = Vec::with_capacity(data.size());
    data.save(&mut serialized_data)?;

    let pb = spinner_with_style();
    pb.set_message("Connecting...");

    let tx = program
        .request()
        .accounts(UpdateAccount {
            candy_guard: candy_guard_id,
            authority: payer.pubkey(),
            payer: payer.pubkey(),
            system_program: system_program::ID,
        })
        .args(Update {
            data: serialized_data,
        });

    let sig = tx.send()?;

    pb.finish_and_clear();
    println!("{} {}", style("Signature:").bold(), sig);

    Ok(())
}
