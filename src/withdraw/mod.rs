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
use anyhow::Result;
use slog::*;
use std::str::FromStr;
use solana_client::{
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_account_decoder::{
    UiAccountEncoding,
};
use mpl_candy_machine::accounts as nft_accounts;
use mpl_candy_machine::instruction as nft_instruction;

use crate::setup::{setup_client, sugar_setup};

pub struct WithdrawArgs {
    pub logger: Logger,
    pub candy_machine: String,
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
}

pub struct WithdrawAllArgs {
    pub logger: Logger,
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
}

pub struct WithdrawSetupConfig {
    pub logger: Logger,
    pub program: Program,
    pub payer: Pubkey,
}


pub fn process_withdraw(args: WithdrawArgs) -> Result<()> {
    let (logger, program, payer) =setup_withdraw(args.logger, args.keypair, args.rpc_url)?;
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


pub fn process_withdraw_all(args: WithdrawAllArgs) -> Result<()> {
 
    let (logger, program, payer) =setup_withdraw(args.logger, args.keypair, args.rpc_url)?;

    let commitment = CommitmentConfig::confirmed();

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
    // info!(
    //     setup.logger,
    //     "Withdrawing funds from all candy machines for key {}", &setup.payer
    // );
    // let sig = program
    //     .request()
    //     .accounts(nft_accounts::WithdrawFunds {
    //         candy_machine,
    //         authority: payer,
    //     })
    //     .args(nft_instruction::WithdrawFunds {})
    //     .send()?;

    // info!(setup.logger, "Transaction submitted with id of: {}", sig);

    let accounts = program.rpc().get_program_accounts_with_config(&program.id(), config)?;
    println!("{:?}", accounts);

    Ok(())
}

fn setup_withdraw (logger:Logger, keypair:Option<String>, rpc_url:Option<String>)->Result<(Logger, Program, Pubkey)>{
    let sugar_config = sugar_setup(logger, keypair, rpc_url)?;

    let logger = sugar_config.logger;
    let client = setup_client(&sugar_config)?;

    let pid = "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ"
        .parse()
        .expect("Failed to parse PID");
    let program = client.program(pid);
    let payer = program.payer();

    Ok((logger,
        program,
        payer))
}