use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use slog::*;
use std::str::FromStr;

use mpl_candy_machine::accounts as nft_accounts;
use mpl_candy_machine::instruction as nft_instruction;

use crate::setup::{setup_client, sugar_setup};

pub struct WithdrawArgs {
    pub logger: Logger,
    pub candy_machine: String,
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
}

pub fn process_withdraw(args: WithdrawArgs) -> Result<()> {
    let sugar_config = sugar_setup(args.logger, args.keypair, args.rpc_url)?;

    let logger = &sugar_config.logger;
    let client = setup_client(&sugar_config)?;

    let pid = "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ"
        .parse()
        .expect("Failed to parse PID");
    let program = client.program(pid);
    let payer = program.payer();
    let candy_machine = Pubkey::from_str(&args.candy_machine)?;

    info!(
        logger,
        "Withdrawing funds from candy machine {}", &candy_machine
    );
    let sig = program
        .request()
        .accounts(nft_accounts::WithdrawFunds {
            candy_machine,
            authority: payer,
        })
        .args(nft_instruction::WithdrawFunds {})
        .send()?;

    info!(logger, "Transaction submitted with id of: {}", sig);

    Ok(())
}
