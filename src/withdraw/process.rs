pub use anchor_client::{
    solana_sdk::{
        commitment_config::{CommitmentConfig, CommitmentLevel},
        native_token::LAMPORTS_PER_SOL,
        pubkey::Pubkey,
        signature::{Keypair, Signature, Signer},
        system_instruction, system_program, sysvar,
        transaction::Transaction,
    },
    Client, Program,
};
use console::style;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use std::{
    io::{stdin, stdout, Write},
    rc::Rc,
    str::FromStr,
};

use mpl_candy_machine::accounts as nft_accounts;
use mpl_candy_machine::instruction as nft_instruction;

use crate::common::*;
use crate::setup::{setup_client, sugar_setup};
use crate::utils::*;

pub struct WithdrawArgs {
    pub candy_machine: Option<String>,
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub list: bool,
}

pub fn process_withdraw(args: WithdrawArgs) -> Result<()> {
    // (1) Setting up connection

    println!(
        "{} {}Initializing connection",
        style("[1/2]").bold().dim(),
        COMPUTER_EMOJI
    );

    let pb = spinner_with_style();
    pb.set_message("Connecting...");

    let (program, payer) = setup_withdraw(args.keypair, args.rpc_url)?;

    pb.finish_with_message("Connected");

    println!(
        "\n{} {}{} funds",
        style("[2/2]").bold().dim(),
        WITHDRAW_EMOJI,
        if args.list { "Listing" } else { "Retrieving" }
    );

    // the --list flag takes precedence; even if a candy machine id is passed
    // as an argument, we will list the candy machines (no draining happens)
    let candy_machine = if args.list { None } else { args.candy_machine };

    // (2) Retrieving data for listing/draining

    match &candy_machine {
        Some(candy_machine) => {
            let candy_machine = match Pubkey::from_str(candy_machine) {
                Ok(candy_machine) => candy_machine,
                Err(_) => {
                    let error = anyhow!("Failed to parse candy machine id: {}", candy_machine);
                    error!("{:?}", error);
                    return Err(error);
                }
            };

            let pb = spinner_with_style();
            pb.set_message("Draining candy machine...");

            do_withdraw(Rc::new(program), candy_machine, payer)?;

            pb.finish_with_message("Done");
        }
        None => {
            let config = RpcProgramAccountsConfig {
                filters: Some(vec![RpcFilterType::Memcmp(Memcmp {
                    offset: 8, // key
                    bytes: MemcmpEncodedBytes::Base58(payer.to_string()),
                    encoding: None,
                })]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    data_slice: None,
                    commitment: Some(CommitmentConfig {
                        commitment: CommitmentLevel::Confirmed,
                    }),
                },
                with_context: None,
            };

            let pb = spinner_with_style();
            pb.set_message("Looking up candy machines...");

            let program = Rc::new(program);
            let accounts = program
                .rpc()
                .get_program_accounts_with_config(&program.id(), config)?;

            pb.finish_and_clear();

            let mut total = 0.0f64;

            accounts.iter().for_each(|account| {
                let (_pubkey, account) = account;
                total += account.lamports as f64;
            });

            println!(
                "Found {} candy machines, total amount: ◎ {}",
                accounts.len(),
                total / LAMPORTS_PER_SOL as f64
            );

            if accounts.is_empty() {
                // nothing else to do, we just say goodbye
                println!("\n{}", style("[Completed]").bold().dim());
            } else if args.list {
                println!("\n{:48} Balance", "Candy Machine ID");
                println!("{:-<61}", "-");

                for (pubkey, account) in accounts {
                    println!(
                        "{:48} {:>12.8}",
                        pubkey.to_string(),
                        account.lamports as f64 / LAMPORTS_PER_SOL as f64
                    );
                }

                println!("\n{}", style("[Completed]").bold().dim());
            } else {
                println!("\n+----------------------------------------------+");
                println!("| WARNING: This will drain all candy machines. |");
                println!("+----------------------------------------------+");

                print!("\nContinue? [Y/n] (default \'n\'): ");
                stdout().flush().ok();

                let mut s = String::new();
                stdin().read_line(&mut s).expect("Error reading input.");

                if let Some('Y') = s.chars().next() {
                    let pb = progress_bar_with_style(accounts.len() as u64);
                    let mut not_drained = 0;

                    accounts.iter().for_each(|account| {
                        let (candy_machine, _account) = account;
                        do_withdraw(program.clone(), *candy_machine, payer).unwrap_or_else(|e| {
                            not_drained += 1;
                            error!("Error: {}", e);
                        });
                        pb.inc(1);
                    });

                    pb.finish();

                    if not_drained > 0 {
                        println!(
                            "{}",
                            style(format!("Could not drain {} candy machine(s)", not_drained))
                                .red()
                                .bold()
                                .dim()
                        );
                    }
                } else {
                    // there were candy machines to drain, but the user decided
                    // to abort the withdraw
                    println!("\n{}", style("Withdraw aborted.").red().bold().dim());
                }
            }
        }
    }

    Ok(())
}

fn setup_withdraw(keypair: Option<String>, rpc_url: Option<String>) -> Result<(Program, Pubkey)> {
    let sugar_config = sugar_setup(keypair, rpc_url)?;

    let client = setup_client(&sugar_config)?;

    let pid = CANDY_MACHINE_V2.parse().expect("Failed to parse PID");
    let program = client.program(pid);
    let payer = program.payer();

    Ok((program, payer))
}

fn do_withdraw(program: Rc<Program>, candy_machine: Pubkey, payer: Pubkey) -> Result<()> {
    program
        .request()
        .accounts(nft_accounts::WithdrawFunds {
            candy_machine,
            authority: payer,
        })
        .args(nft_instruction::WithdrawFunds {})
        .send()?;

    Ok(())
}
