use anchor_client::{
    solana_sdk::{
        instruction::Instruction,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        system_instruction, system_program, sysvar,
    },
    Client,
};
use anchor_lang::prelude::AccountMeta;
use anyhow::Result;
use mpl_token_metadata::ID as TOKEN_METADATA_ID;
use rand::rngs::OsRng;
use slog::*;
use spl_associated_token_account::{
    create_associated_token_account, get_associated_token_address,
    ID as ASSOCIATED_TOKEN_PROGRAM_ID,
};
use spl_token::{
    instruction::{initialize_mint, mint_to},
    ID as TOKEN_PROGRAM_ID,
};
use std::{fs::File, path::Path, str::FromStr};

use mpl_candy_machine::accounts as nft_accounts;
use mpl_candy_machine::instruction as nft_instruction;
use mpl_candy_machine::{CandyMachine, WhitelistMintMode, ID as CANDY_MACHINE_PROGRAM_ID};

use crate::cache::Cache;
use crate::candy_machine::*;
use crate::constants::*;
use crate::setup::*;

const MINT_LAYOUT: u64 = 82;

pub struct MintOneArgs {
    pub logger: Logger,
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
}

pub fn process_mint_one(mint_args: MintOneArgs) -> Result<()> {
    let sugar_config = sugar_setup(mint_args.logger, mint_args.keypair, mint_args.rpc_url)?;

    let cache: Cache = if Path::new("cache.json").exists() {
        let file = File::open("cache.json")?;
        serde_json::from_reader(file)?
    } else {
        error!(sugar_config.logger, "cache.json does not exist");
        std::process::exit(1);
    };

    let client = setup_client(&sugar_config)?;

    let candy_machine_id = match Pubkey::from_str(&cache.program.candy_machine) {
        Ok(candy_machine_id) => candy_machine_id,
        Err(_) => {
            error!(
                sugar_config.logger,
                "Failed to parse candy_machine_id: {}", &cache.program.candy_machine
            );
            std::process::exit(1);
        }
    };

    let candy_machine_state = get_candy_machine_state(&sugar_config, &candy_machine_id)?;

    info!(
        sugar_config.logger,
        "Minting NFT from candy machine: {}", &candy_machine_id
    );
    info!(
        sugar_config.logger,
        "Candy machine program id: {:?}", CANDY_MACHINE_PROGRAM_ID
    );
    mint_one(
        sugar_config.logger,
        client,
        candy_machine_id,
        candy_machine_state,
    )?;

    Ok(())
}

pub fn mint_one(
    logger: Logger,
    client: Client,
    candy_machine_id: Pubkey,
    candy_machine_state: CandyMachine,
) -> Result<()> {
    let pid = "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ"
        .parse()
        .expect("Failed to parse PID");

    let program = client.program(pid);
    let payer = program.payer();
    let wallet = candy_machine_state.wallet;

    let candy_machine_data = candy_machine_state.data;

    let mint = Keypair::new();
    let metaplex_program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;

    // Allocate memory for the account
    let min_rent = program
        .rpc()
        .get_minimum_balance_for_rent_exemption(MINT_LAYOUT as usize)?;

    // Create mint account
    let create_mint_account_ix = system_instruction::create_account(
        &payer,
        &mint.pubkey(),
        min_rent,
        MINT_LAYOUT,
        &TOKEN_PROGRAM_ID,
    );

    // Initalize mint ix
    let init_mint_ix = initialize_mint(&TOKEN_PROGRAM_ID, &mint.pubkey(), &payer, Some(&payer), 0)?;

    // Derive associated token account
    let assoc = get_associated_token_address(&payer, &mint.pubkey());

    // Create associated account instruction
    let create_assoc_account_ix = create_associated_token_account(&payer, &payer, &mint.pubkey());

    // Mint to instruction
    let mint_to_ix = mint_to(&TOKEN_PROGRAM_ID, &mint.pubkey(), &assoc, &payer, &[], 1)?;

    // Check gatekeeper
    // if let Some(gatekeeper) = candy_machine_data.gateeker {

    // }

    let mut additional_instructions: Vec<Instruction> = Vec::new();
    let mut additional_accounts: Vec<AccountMeta> = Vec::new();
    let mut additional_signers: Vec<Keypair> = Vec::new();

    // Check whitelist mint settings
    if let Some(mint_settings) = candy_machine_data.whitelist_mint_settings {
        let whitelist_token = get_ata_for_mint(&mint_settings.mint, &payer);

        println!("Whitelist token {:?}", whitelist_token);

        additional_accounts.push(AccountMeta {
            pubkey: whitelist_token,
            is_signer: false,
            is_writable: true,
        });

        if mint_settings.mode == WhitelistMintMode::BurnEveryTime {
            let whitelist_burn_authority = Keypair::generate(&mut OsRng);

            println!(
                "whitelist burn auth: {:?}",
                &whitelist_burn_authority.pubkey()
            );

            println!("whitelist mint: {:?}", &mint_settings.mint);

            let ata_exists = !program.rpc().get_account_data(&whitelist_token)?.is_empty();

            if ata_exists {
                let approve_ix = spl_token::instruction::approve(
                    &TOKEN_PROGRAM_ID,
                    &whitelist_token,
                    &whitelist_burn_authority.pubkey(),
                    &payer,
                    &[],
                    1,
                )?;
                let revoke_ix = spl_token::instruction::revoke(
                    &TOKEN_PROGRAM_ID,
                    &whitelist_token,
                    &payer,
                    &[],
                )?;

                additional_accounts.push(AccountMeta {
                    pubkey: mint.pubkey(),
                    is_signer: false,
                    is_writable: true,
                });
                additional_accounts.push(AccountMeta {
                    pubkey: whitelist_burn_authority.pubkey(),
                    is_signer: true,
                    is_writable: false,
                });

                additional_instructions.push(approve_ix);
                additional_instructions.push(revoke_ix);

                additional_signers.push(whitelist_burn_authority);
            }
        }
    }

    // Check token mint

    let metadata_pda = get_metadata_pda(&mint.pubkey())?;
    let master_edition_pda = get_master_edition_pda(&mint.pubkey())?;
    let (candy_machine_creator_pda, creator_bump) =
        get_candy_machine_creator_pda(&candy_machine_id)?;

    let mut builder = program
        .request()
        .instruction(create_mint_account_ix)
        .instruction(init_mint_ix)
        .instruction(create_assoc_account_ix)
        .instruction(mint_to_ix)
        .signer(&mint)
        .accounts(nft_accounts::MintNFT {
            candy_machine: candy_machine_id,
            candy_machine_creator: candy_machine_creator_pda,
            payer,
            wallet,
            metadata: metadata_pda,
            mint: mint.pubkey(),
            mint_authority: payer,
            update_authority: payer,
            master_edition: master_edition_pda,
            token_metadata_program: metaplex_program_id,
            token_program: TOKEN_PROGRAM_ID,
            system_program: system_program::id(),
            rent: sysvar::rent::ID,
            clock: sysvar::clock::ID,
            recent_blockhashes: sysvar::recent_blockhashes::ID,
            instruction_sysvar_account: sysvar::instructions::ID,
        })
        .args(nft_instruction::MintNft { creator_bump });

    // Add additional instructions based on candy machine settings.
    if !additional_instructions.is_empty() {
        for instruction in additional_instructions {
            builder = builder.instruction(instruction);
        }
    }

    if !additional_accounts.is_empty() {
        for account in additional_accounts {
            builder = builder.accounts(account);
        }
    }

    if !additional_signers.is_empty() {
        for signer in &additional_signers {
            builder = builder.signer(signer);
        }
    }

    let sig = builder.send()?;

    info!(logger, "Minted! TxId: {}", sig);
    Ok(())
}

fn _get_network_token(gatekeeper: &Pubkey, payer: &Pubkey) -> Result<Pubkey> {
    let civic = Pubkey::from_str(CIVIC)?;
    let seeds = &[
        &payer.to_bytes(),
        "gateway".as_bytes(),
        &[0, 0, 0, 0, 0, 0, 0, 0],
        &gatekeeper.to_bytes(),
    ];
    let (pda, _bump) = Pubkey::find_program_address(seeds, &civic);

    Ok(pda)
}

fn _get_network_expire(gatekeeper: &Pubkey) -> Result<Pubkey> {
    let civic = Pubkey::from_str(CIVIC)?;
    let seeds = &[&gatekeeper.to_bytes(), "expire".as_bytes()];
    let (pda, _bump) = Pubkey::find_program_address(seeds, &civic);

    Ok(pda)
}

fn get_ata_for_mint(mint: &Pubkey, buyer: &Pubkey) -> Pubkey {
    let seeds: &[&[u8]] = &[
        &buyer.to_bytes(),
        &TOKEN_PROGRAM_ID.to_bytes(),
        &mint.to_bytes(),
    ];
    let (pda, _bump) = Pubkey::find_program_address(seeds, &ASSOCIATED_TOKEN_PROGRAM_ID);
    pda
}

fn get_metadata_pda(mint: &Pubkey) -> Result<Pubkey> {
    // Derive metadata account
    let metadata_seeds = &[
        "metadata".as_bytes(),
        &TOKEN_METADATA_ID.to_bytes(),
        &mint.to_bytes(),
    ];
    let (pda, _bump) = Pubkey::find_program_address(metadata_seeds, &TOKEN_METADATA_ID);

    Ok(pda)
}

fn get_master_edition_pda(mint: &Pubkey) -> Result<Pubkey> {
    // Derive Master Edition account
    let master_edition_seeds = &[
        "metadata".as_bytes(),
        &TOKEN_METADATA_ID.to_bytes(),
        &mint.to_bytes(),
        "edition".as_bytes(),
    ];
    let (pda, _bump) = Pubkey::find_program_address(master_edition_seeds, &TOKEN_METADATA_ID);

    Ok(pda)
}

fn get_candy_machine_creator_pda(candy_machine_id: &Pubkey) -> Result<(Pubkey, u8)> {
    // Derive metadata account
    let creator_seeds = &["candy_machine".as_bytes(), &candy_machine_id.as_ref()];

    Ok(Pubkey::find_program_address(
        creator_seeds,
        &CANDY_MACHINE_PROGRAM_ID,
    ))
}
