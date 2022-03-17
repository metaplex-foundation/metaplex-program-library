pub use anchor_client::{
    solana_sdk::{
        commitment_config::{CommitmentConfig, CommitmentLevel},
        pubkey::Pubkey,
        signature::{Keypair, Signature, Signer},
        system_instruction, system_program, sysvar,
        transaction::Transaction,
    },
    Client, Program,
};
use indicatif::ProgressBar;
use mpl_candy_machine::accounts as nft_accounts;
use mpl_candy_machine::instruction as nft_instruction;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use std::{rc::Rc, str::FromStr, thread, time::Duration};

use crate::common::*;
use crate::setup::{setup_client, sugar_setup};

pub struct WithdrawArgs {
    pub candy_machine: String,
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
}

pub struct WithdrawAllArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
}

pub struct WithdrawSetupConfig {
    pub program: Program,
    pub payer: Pubkey,
}

pub fn process_withdraw(args: WithdrawArgs) -> Result<()> {
    let (program, payer) = setup_withdraw(args.keypair, args.rpc_url)?;
    let candy_machine = match Pubkey::from_str(&args.candy_machine) {
        Ok(candy_machine) => candy_machine,
        Err(_) => {
            let error = anyhow!("Failed to parse candy machine id: {}", args.candy_machine);
            error!("{:?}", error);
            return Err(error);
        }
    };

    let program = Rc::new(program);
    do_withdraw(Rc::clone(&program), candy_machine, payer)?;
    Ok(())
}

pub fn process_withdraw_all(args: WithdrawAllArgs) -> Result<()> {
    let (program, payer) = setup_withdraw(args.keypair, args.rpc_url)?;

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

    let program = Rc::new(program);
    let accounts = program
        .rpc()
        .get_program_accounts_with_config(&program.id(), config)?;

    let pb = ProgressBar::new(accounts.len() as u64);

    accounts.iter().for_each(|account| {
        let (candy_machine, _account) = account;
        match do_withdraw(program.clone(), *candy_machine, payer) {
            Ok(_) => {
                pb.inc(1);
            }
            Err(e) => {
                error!("Error: {}", e);
                pb.inc(1);
                return;
            }
        }
    });
    thread::sleep(Duration::from_millis(1000));

    Ok(())
}

fn setup_withdraw(keypair: Option<String>, rpc_url: Option<String>) -> Result<(Program, Pubkey)> {
    let sugar_config = sugar_setup(keypair, rpc_url)?;

    let client = setup_client(&sugar_config)?;

    let pid = "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ"
        .parse()
        .expect("Failed to parse PID");
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
