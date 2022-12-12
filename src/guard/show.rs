use std::str::FromStr;

use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use chrono::NaiveDateTime;
use console::style;
use mpl_candy_guard::state::{CandyGuard, CandyGuardData, GuardSet, DATA_OFFSET};
use mpl_candy_machine_core::constants::EMPTY_STR;
use solana_program::native_token::LAMPORTS_PER_SOL;

use crate::{cache::load_cache, common::*, show::print_with_style, utils::*};

pub struct GuardShowArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub candy_guard: Option<String>,
}

pub fn process_guard_show(args: GuardShowArgs) -> Result<()> {
    println!("[1/1] {}Loading candy guard", LOOKING_GLASS_EMOJI);

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

    let pb = spinner_with_style();
    pb.set_message("Connecting...");

    let account: CandyGuard = program.account(candy_guard_id)?;
    let account_data = program.rpc().get_account_data(&candy_guard_id)?;
    // load the guard set information
    let candy_guard_data = CandyGuardData::load(&account_data[DATA_OFFSET..])?;

    pb.finish_with_message("Done");

    println!(
        "\n{}{} {}",
        GUARD_EMOJI,
        style("Candy Guard ID:").dim(),
        &candy_guard_id
    );

    // candy guard configuration

    println!(" {}", style(":").dim());
    print_with_style("", "base", account.base.to_string());
    print_with_style("", "bump", account.bump.to_string());
    print_with_style("", "authority", account.authority.to_string());
    print_with_style("", "data", EMPTY_STR.to_string());

    // default guard set
    print_with_style("    ", "default", EMPTY_STR.to_string());
    print_guard_set(&candy_guard_data.default, "    :   ".to_string())?;

    // groups
    if let Some(groups) = candy_guard_data.groups {
        println!("     {}", style(":").dim());
        print_with_style("    ", "groups", EMPTY_STR.to_string());

        for (index, group) in groups.iter().enumerate() {
            if index > 0 {
                // padding between groups
                println!("          {}", style(":").dim());
            }
            print_with_style("         ", "label", &group.label);
            print_guard_set(
                &group.guards,
                if index == (groups.len() - 1) {
                    "             ".to_string()
                } else {
                    "         :   ".to_string()
                },
            )?;
        }
    } else {
        print_with_style("    ", "groups", "none".to_string());
    }

    Ok(())
}

fn print_guard_set(guard_set: &GuardSet, padding: String) -> Result<()> {
    // bot tax
    if let Some(bot_tax) = &guard_set.bot_tax {
        print_with_style(&padding, "bot tax", EMPTY_STR.to_string());
        print_with_style(
            &format!("{}:   ", padding),
            "lamports",
            format!(
                "{} (◎ {})",
                bot_tax.lamports,
                bot_tax.lamports as f64 / LAMPORTS_PER_SOL as f64
            ),
        );
        print_with_style(
            &format!("{}:   ", padding),
            "last instruction",
            bot_tax.last_instruction.to_string(),
        );
    } else {
        print_with_style(&padding, "bot tax", "none".to_string());
    }

    // sol payment
    if let Some(sol_payment) = &guard_set.sol_payment {
        print_with_style(&padding, "sol payment", EMPTY_STR.to_string());
        print_with_style(
            &format!("{}:   ", padding),
            "lamports",
            format!(
                "{} (◎ {})",
                sol_payment.lamports,
                sol_payment.lamports as f64 / LAMPORTS_PER_SOL as f64
            ),
        );
        print_with_style(
            &format!("{}:   ", padding),
            "destination",
            sol_payment.destination.to_string(),
        );
    } else {
        print_with_style(&padding, "sol payment", "none".to_string());
    }

    // token payment
    if let Some(token_payment) = &guard_set.token_payment {
        print_with_style(&padding, "token payment", EMPTY_STR.to_string());
        print_with_style(
            &format!("{}:   ", padding),
            "amount",
            token_payment.amount.to_string(),
        );
        print_with_style(
            &format!("{}:   ", padding),
            "token mint",
            token_payment.mint.to_string(),
        );
        print_with_style(
            &format!("{}:   ", padding),
            "destination",
            token_payment.destination_ata.to_string(),
        );
    } else {
        print_with_style(&padding, "token payment", "none".to_string());
    }

    // start date
    if let Some(start_date) = &guard_set.start_date {
        print_with_style(&padding, "start date", EMPTY_STR.to_string());
        if let Some(date) = NaiveDateTime::from_timestamp_opt(start_date.date, 0) {
            print_with_style(
                &format!("{}:   ", padding),
                "date",
                date.format("%a %B %e %Y %H:%M:%S UTC").to_string(),
            );
        } else {
            // this should not happen, but adding a message so it can be
            // flag to the user
            print_with_style(&format!("{}:   ", padding), "date", "<parse error>");
        }
    } else {
        print_with_style(&padding, "start date", "none".to_string());
    }

    // third party signer
    if let Some(third_party_signer) = &guard_set.third_party_signer {
        print_with_style(&padding, "third party signer", EMPTY_STR.to_string());
        print_with_style(
            &format!("{}:   ", padding),
            "signer key",
            third_party_signer.signer_key.to_string(),
        );
    } else {
        print_with_style(&padding, "third party signer", "none".to_string());
    }

    // token gate
    if let Some(token_gate) = &guard_set.token_gate {
        print_with_style(&padding, "token gate", EMPTY_STR.to_string());
        print_with_style(
            &format!("{}:   ", padding),
            "amount",
            token_gate.amount.to_string(),
        );
        print_with_style(
            &format!("{}:   ", padding),
            "mint",
            token_gate.mint.to_string(),
        );
    } else {
        print_with_style(&padding, "token gate", "none".to_string());
    }

    // gatekeeper
    if let Some(gatekeeper) = &guard_set.gatekeeper {
        print_with_style(&padding, "gatekeeper", EMPTY_STR.to_string());
        print_with_style(
            &format!("{}:   ", padding),
            "gatekeeper network",
            gatekeeper.gatekeeper_network.to_string(),
        );
        print_with_style(
            &format!("{}:   ", padding),
            "expire_on_use",
            gatekeeper.expire_on_use.to_string(),
        );
    } else {
        print_with_style(&padding, "gatekeeper", "none".to_string());
    }

    // end date
    if let Some(end_date) = &guard_set.end_date {
        print_with_style(&padding, "end date", EMPTY_STR.to_string());
        if let Some(date) = NaiveDateTime::from_timestamp_opt(end_date.date, 0) {
            print_with_style(
                &format!("{}:   ", padding),
                "date",
                date.format("%a %B %e %Y %H:%M:%S UTC").to_string(),
            );
        } else {
            // this should not happen, but adding a message so it can be
            // flag to the user
            print_with_style(&format!("{}:   ", padding), "date", "<parse error>");
        }
    } else {
        print_with_style(&padding, "end date", "none".to_string());
    }

    // allow list
    if let Some(allow_list) = &guard_set.allow_list {
        print_with_style(&padding, "allow list", EMPTY_STR.to_string());
        print_with_style(
            &format!("{}:   ", padding),
            "merkle root",
            hex::encode(allow_list.merkle_root),
        );
    } else {
        print_with_style(&padding, "allow list", "none".to_string());
    }

    // mint limit
    if let Some(mint_limit) = &guard_set.mint_limit {
        print_with_style(&padding, "mint limit", EMPTY_STR.to_string());
        print_with_style(&format!("{}:   ", padding), "id", mint_limit.id.to_string());
        print_with_style(
            &format!("{}:   ", padding),
            "amount",
            mint_limit.limit.to_string(),
        );
    } else {
        print_with_style(&padding, "mint limit", "none".to_string());
    }

    // nft payment
    if let Some(nft_payment) = &guard_set.nft_payment {
        print_with_style(&padding, "nft payment", EMPTY_STR.to_string());
        print_with_style(
            &format!("{}:   ", padding),
            "required collection",
            nft_payment.required_collection.to_string(),
        );
        print_with_style(
            &format!("{}:   ", padding),
            "destination",
            nft_payment.destination.to_string(),
        );
    } else {
        print_with_style(&padding, "nft payment", "none".to_string());
    }

    // redeemed amount
    if let Some(redeemed_amount) = &guard_set.redeemed_amount {
        print_with_style(&padding, "redeemed amount", EMPTY_STR.to_string());
        print_with_style(
            &format!("{}:   ", padding),
            "amount",
            redeemed_amount.maximum.to_string(),
        );
    } else {
        print_with_style(&padding, "redeemed amount", "none".to_string());
    }

    // address gate
    if let Some(address_gate) = &guard_set.address_gate {
        print_with_style(&padding, "address gate", EMPTY_STR.to_string());
        print_with_style(
            &format!("{}:   ", padding),
            "address",
            address_gate.address.to_string(),
        );
    } else {
        print_with_style(&padding, "address gate", "none".to_string());
    }

    // nft gate
    if let Some(nft_gate) = &guard_set.nft_gate {
        print_with_style(&padding, "nft gate", EMPTY_STR.to_string());
        print_with_style(
            &format!("{}:   ", padding),
            "required_collection",
            nft_gate.required_collection.to_string(),
        );
    } else {
        print_with_style(&padding, "nft gate", "none".to_string());
    }

    // nft burn
    if let Some(nft_burn) = &guard_set.nft_burn {
        print_with_style(&padding, "nft burn", EMPTY_STR.to_string());
        print_with_style(
            &format!("{}:   ", padding),
            "required_collection",
            nft_burn.required_collection.to_string(),
        );
    } else {
        print_with_style(&padding, "nft burn", "none".to_string());
    }

    // token burn
    if let Some(token_burn) = &guard_set.token_burn {
        print_with_style(&padding, "token burn", EMPTY_STR.to_string());
        print_with_style(
            &format!("{}    ", padding),
            "amount",
            token_burn.amount.to_string(),
        );
        print_with_style(
            &format!("{}    ", padding),
            "mint",
            token_burn.mint.to_string(),
        );
    } else {
        print_with_style(&padding, "token burn", "none".to_string());
    }

    // freeze sol payment
    if let Some(freeze_sol_payment) = &guard_set.freeze_sol_payment {
        print_with_style(&padding, "freeze sol payment", EMPTY_STR.to_string());
        print_with_style(
            &format!("{}:   ", padding),
            "lamports",
            format!(
                "{} (◎ {})",
                freeze_sol_payment.lamports,
                freeze_sol_payment.lamports as f64 / LAMPORTS_PER_SOL as f64
            ),
        );
        print_with_style(
            &format!("{}:   ", padding),
            "destination",
            freeze_sol_payment.destination.to_string(),
        );
    } else {
        print_with_style(&padding, "freeze sol payment", "none".to_string());
    }

    // freeze token payment
    if let Some(freeze_token_payment) = &guard_set.freeze_token_payment {
        print_with_style(&padding, "freeze token payment", EMPTY_STR.to_string());
        print_with_style(
            &format!("{}:   ", padding),
            "amount",
            freeze_token_payment.amount.to_string(),
        );
        print_with_style(
            &format!("{}:   ", padding),
            "token mint",
            freeze_token_payment.mint.to_string(),
        );
        print_with_style(
            &format!("{}:   ", padding),
            "destination",
            freeze_token_payment.destination_ata.to_string(),
        );
    } else {
        print_with_style(&padding, "freeze token payment", "none".to_string());
    }

    Ok(())
}
