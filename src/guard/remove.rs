use std::str::FromStr;

use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use console::style;
use mpl_candy_guard::{accounts::Unwrap as UnwrapAccount, instruction::Unwrap};

use crate::{cache::load_cache, candy_machine::*, common::*, utils::*};

pub struct GuardRemoveArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub candy_machine: Option<String>,
    pub candy_guard: Option<String>,
}

pub fn process_guard_remove(args: GuardRemoveArgs) -> Result<()> {
    println!("[1/1] {}Unwrapping", UNWRAP_EMOJI);

    // the candy machine id specified takes precedence over the one from the cache

    let candy_machine_id = if let Some(candy_machine) = args.candy_machine {
        candy_machine
    } else {
        let cache = load_cache(&args.cache, false)?;
        cache.program.candy_machine
    };

    let candy_machine_id = match Pubkey::from_str(&candy_machine_id) {
        Ok(candy_machine_id) => candy_machine_id,
        Err(_) => {
            let error = anyhow!("Failed to parse candy machine id: {}", candy_machine_id);
            error!("{:?}", error);
            return Err(error);
        }
    };

    // the candy guard id specified takes precedence over the one from the cache

    let candy_guard_id = if let Some(candy_guard) = args.candy_guard {
        candy_guard
    } else {
        let cache = load_cache(&args.cache, false)?;
        cache.program.candy_guard
    };

    let candy_guard_id = match Pubkey::from_str(&candy_guard_id) {
        Ok(candy_guard_id) => candy_guard_id,
        Err(_) => {
            let error = anyhow!("Failed to parse candy guard id: {}", candy_guard_id);
            error!("{:?}", error);
            return Err(error);
        }
    };

    // remove the candy guard as mint authority

    let sugar_config = sugar_setup(args.keypair, args.rpc_url)?;
    let client = setup_client(&sugar_config)?;
    let program = client.program(mpl_candy_guard::ID);
    let payer = sugar_config.keypair;

    let pb = spinner_with_style();
    pb.set_message("Connecting...");

    let tx = program
        .request()
        .accounts(UnwrapAccount {
            candy_guard: candy_guard_id,
            authority: payer.pubkey(),
            candy_machine: candy_machine_id,
            candy_machine_authority: payer.pubkey(),
            candy_machine_program: CANDY_MACHINE_ID,
        })
        .args(Unwrap {});

    let sig = tx.send()?;

    pb.finish_and_clear();
    println!("{} {}", style("Signature:").bold(), sig);

    println!("\nThe candy guard is no longer the mint authority of the candy machine.");
    println!(
        "  -> New mint authority: {}",
        style(format!("{}", payer.pubkey())).bold()
    );

    Ok(())
}
