use std::str::FromStr;

use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use console::style;
use mpl_candy_guard::{
    accounts::{Initialize as InitializeAccount, Update as UpdateAccount, Wrap as WrapAccount},
    instruction::{Initialize, Update, Wrap},
};
use mpl_candy_machine_core::constants::EMPTY_STR;

use crate::{cache::load_cache, candy_machine::*, common::*, config::get_config_data, utils::*};

pub struct GuardAddArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub config: String,
    pub candy_machine: Option<String>,
    pub candy_guard: Option<String>,
}

pub fn process_guard_add(args: GuardAddArgs) -> Result<()> {
    println!("[1/3] {}Looking up candy machine", LOOKING_GLASS_EMOJI);

    let pb = spinner_with_style();
    pb.set_message("Connecting...");

    // the candy machine id specified takes precedence over the one from the cache

    let (candy_machine_id, cache) = if let Some(candy_machine) = args.candy_machine {
        (candy_machine, None)
    } else {
        let cache = load_cache(&args.cache, false)?;
        (cache.program.candy_machine.clone(), Some(cache))
    };

    if candy_machine_id.is_empty() {
        return Err(anyhow!("Missing candy machine id."));
    }

    let candy_machine_id = match Pubkey::from_str(&candy_machine_id) {
        Ok(candy_machine_id) => candy_machine_id,
        Err(_) => {
            let error = anyhow!("Failed to parse candy machine id: {}", candy_machine_id);
            error!("{:?}", error);
            return Err(error);
        }
    };

    pb.finish_and_clear();

    println!(
        "\n{} {}",
        style("Candy machine ID:").bold(),
        candy_machine_id
    );

    // decide whether to create a new candy guard or use an existing one

    let candy_guard_id = if let Some(candy_guard) = args.candy_guard {
        candy_guard
    } else if let Some(ref cache) = cache {
        cache.program.candy_guard.clone()
    } else {
        EMPTY_STR.to_string()
    };

    let sugar_config = sugar_setup(args.keypair, args.rpc_url)?;
    let config_data = get_config_data(&args.config)?;
    let client = setup_client(&sugar_config)?;
    let payer = sugar_config.keypair;
    let program = client.program(mpl_candy_guard::ID);

    let candy_guard = if candy_guard_id.is_empty() {
        println!("\n[2/3] {}Initializing a candy guard", GUARD_EMOJI);
        let pb = spinner_with_style();
        pb.set_message("Initializing...");

        let data = if let Some(guards) = &config_data.guards {
            guards.to_guard_format()?
        } else {
            return Err(anyhow!("Missing guards configuration."));
        };

        let base = Keypair::new();
        let (candy_guard, _) = Pubkey::find_program_address(
            &[b"candy_guard", base.pubkey().as_ref()],
            &mpl_candy_guard::ID,
        );

        let tx = program
            .request()
            .accounts(InitializeAccount {
                candy_guard,
                base: base.pubkey(),
                authority: payer.pubkey(),
                payer: payer.pubkey(),
                system_program: system_program::id(),
            })
            .args(Initialize { data })
            .signer(&base);

        let sig = tx.send()?;

        pb.finish_and_clear();
        println!("{} {}", style("Signature:").bold(), sig);

        candy_guard
    } else {
        println!("\n[2/3] {}Loading candy guard", COMPUTER_EMOJI);

        let candy_guard_id = match Pubkey::from_str(&candy_guard_id) {
            Ok(candy_guard_id) => candy_guard_id,
            Err(_) => {
                let error = anyhow!("Failed to parse candy guard id: {}", candy_guard_id);
                error!("{:?}", error);
                return Err(error);
            }
        };

        let pb = spinner_with_style();
        pb.set_message("Connecting...");

        // validates that the account exists
        let _candy_guard = program.rpc().get_account(&candy_guard_id)?;

        let data = if let Some(guards) = &config_data.guards {
            guards.to_guard_format()?
        } else {
            return Err(anyhow!("Missing guards configuration."));
        };

        // synchronizes the guards config with the on-chain account
        let tx = program
            .request()
            .accounts(UpdateAccount {
                candy_guard: candy_guard_id,
                authority: payer.pubkey(),
                payer: payer.pubkey(),
                system_program: system_program::ID,
            })
            .args(Update { data });

        tx.send()?;

        pb.finish_with_message("Done");

        candy_guard_id
    };

    println!("\n{} {}", style("Candy guard ID:").bold(), candy_guard);

    // wraps the candy machine

    println!("\n[3/3] {}Wrapping", WRAP_EMOJI);

    let pb = spinner_with_style();
    pb.set_message("Connecting...");

    let tx = program
        .request()
        .accounts(WrapAccount {
            candy_guard,
            authority: payer.pubkey(),
            candy_machine: candy_machine_id,
            candy_machine_program: CANDY_MACHINE_ID,
            candy_machine_authority: payer.pubkey(),
        })
        .args(Wrap {});

    let sig = tx.send()?;

    pb.finish_and_clear();
    println!("{} {}", style("Signature:").bold(), sig);

    println!("\nThe candy guard is now the mint authority of the candy machine.");

    // if we created a new candy guard from the candy machine on the cache file,
    // we store the reference of the candy guard on the cache

    if cache.is_some() {
        let mut cache = load_cache(&args.cache, false)?;
        cache.program.candy_guard = candy_guard.to_string();
        cache.sync_file()?;
    }

    Ok(())
}
