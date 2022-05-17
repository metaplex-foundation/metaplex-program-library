use anchor_client::{
    solana_sdk::{
        program_pack::Pack,
        pubkey::Pubkey,
        signature::{Keypair, Signature, Signer},
        system_instruction, system_program, sysvar,
    },
    Client,
};
use anchor_lang::prelude::AccountMeta;
use anyhow::Result;
use chrono::Utc;
use console::style;
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
use spl_token::{
    instruction::{initialize_mint, mint_to},
    state::Account,
    ID as TOKEN_PROGRAM_ID,
};
use std::{str::FromStr, sync::Arc};

use mpl_candy_machine::accounts as nft_accounts;
use mpl_candy_machine::instruction as nft_instruction;
use mpl_candy_machine::{CandyMachine, EndSettingType, ErrorCode, WhitelistMintMode};

use crate::cache::load_cache;
use crate::candy_machine::ID as CANDY_MACHINE_ID;
use crate::candy_machine::*;
use crate::common::*;
use crate::mint::pdas::*;
use crate::utils::*;

pub struct MintArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub number: Option<u64>,
    pub candy_machine: Option<String>,
}

pub fn process_mint(args: MintArgs) -> Result<()> {
    let sugar_config = sugar_setup(args.keypair, args.rpc_url)?;
    let client = Arc::new(setup_client(&sugar_config)?);

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
        "{} {}Minting from candy machine",
        style("[1/1]").bold().dim(),
        CANDY_EMOJI
    );
    println!("Candy machine ID: {}", &candy_machine_id);

    let candy_machine_state = Arc::new(get_candy_machine_state(&sugar_config, &candy_pubkey)?);
    let number = args.number.unwrap_or(1);
    let available = candy_machine_state.data.items_available - candy_machine_state.items_redeemed;

    if number > available || number == 0 {
        let error = anyhow!("{} item(s) available, requested {}", available, number);
        error!("{:?}", error);
        return Err(error);
    }

    info!("Minting NFT from candy machine: {}", &candy_machine_id);
    info!("Candy machine program id: {:?}", CANDY_MACHINE_ID);

    if number == 1 {
        let pb = spinner_with_style();
        pb.set_message(format!(
            "{} item(s) remaining",
            candy_machine_state.data.items_available - candy_machine_state.items_redeemed
        ));

        let result = match mint(
            Arc::clone(&client),
            candy_pubkey,
            Arc::clone(&candy_machine_state),
        ) {
            Ok(signature) => format!("{} {}", style("Signature:").bold(), signature),
            Err(err) => {
                pb.abandon_with_message(format!("{}", style("Mint failed ").red().bold()));
                error!("{:?}", err);
                return Err(err);
            }
        };

        pb.finish_with_message(result);
    } else {
        let pb = progress_bar_with_style(number);

        for _i in 0..number {
            if let Err(err) = mint(
                Arc::clone(&client),
                candy_pubkey,
                Arc::clone(&candy_machine_state),
            ) {
                pb.abandon_with_message(format!("{}", style("Mint failed ").red().bold()));
                error!("{:?}", err);
                return Err(err);
            }

            pb.inc(1);
        }

        pb.finish();
    }

    Ok(())
}

pub fn mint(
    client: Arc<Client>,
    candy_machine_id: Pubkey,
    candy_machine_state: Arc<CandyMachine>,
) -> Result<Signature> {
    let program = client.program(CANDY_MACHINE_ID);
    let payer = program.payer();
    let wallet = candy_machine_state.wallet;

    let candy_machine_data = &candy_machine_state.data;

    if let Some(_gatekeeper) = &candy_machine_data.gatekeeper {
        return Err(anyhow!(
            "Command-line mint disabled (gatekeeper settings in use)"
        ));
    } else if candy_machine_state.items_redeemed >= candy_machine_data.items_available {
        return Err(anyhow!(ErrorCode::CandyMachineEmpty));
    }

    if candy_machine_state.authority != payer {
        // we are not authority, we need to follow the rules
        // 1. go_live_date
        // 2. whitelist mint settings
        // 3. end settings
        let mint_date = Utc::now().timestamp();
        let mut mint_enabled = if let Some(date) = candy_machine_data.go_live_date {
            // mint will be enabled only if the go live date is earlier
            // than the current date
            date < mint_date
        } else {
            // this is the case that go live date is null
            false
        };

        if let Some(wl_mint_settings) = &candy_machine_data.whitelist_mint_settings {
            if wl_mint_settings.presale {
                // we (temporarily) enable the mint - we will validate if the user
                // has the wl token when creating the transaction
                mint_enabled = true;
            } else if !mint_enabled {
                return Err(anyhow!(ErrorCode::CandyMachineNotLive));
            }
        }

        if !mint_enabled {
            // no whitelist mint settings (or no presale) and we are earlier than
            // go live date
            return Err(anyhow!(ErrorCode::CandyMachineNotLive));
        }

        if let Some(end_settings) = &candy_machine_data.end_settings {
            match end_settings.end_setting_type {
                EndSettingType::Date => {
                    if (end_settings.number as i64) < mint_date {
                        return Err(anyhow!(ErrorCode::CandyMachineNotLive));
                    }
                }
                EndSettingType::Amount => {
                    if candy_machine_state.items_redeemed >= end_settings.number {
                        return Err(anyhow!(
                            "Candy machine is not live (end settings amount reached)"
                        ));
                    }
                }
            }
        }
    }

    let nft_mint = Keypair::new();
    let metaplex_program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;

    // Allocate memory for the account
    let min_rent = program
        .rpc()
        .get_minimum_balance_for_rent_exemption(MINT_LAYOUT as usize)?;

    // Create mint account
    let create_mint_account_ix = system_instruction::create_account(
        &payer,
        &nft_mint.pubkey(),
        min_rent,
        MINT_LAYOUT,
        &TOKEN_PROGRAM_ID,
    );

    // Initialize mint ix
    let init_mint_ix = initialize_mint(
        &TOKEN_PROGRAM_ID,
        &nft_mint.pubkey(),
        &payer,
        Some(&payer),
        0,
    )?;

    // Derive associated token account
    let assoc = get_associated_token_address(&payer, &nft_mint.pubkey());

    // Create associated account instruction
    let create_assoc_account_ix =
        create_associated_token_account(&payer, &payer, &nft_mint.pubkey());

    // Mint to instruction
    let mint_to_ix = mint_to(
        &TOKEN_PROGRAM_ID,
        &nft_mint.pubkey(),
        &assoc,
        &payer,
        &[],
        1,
    )?;

    let mut additional_accounts: Vec<AccountMeta> = Vec::new();

    // Check whitelist mint settings
    if let Some(wl_mint_settings) = &candy_machine_data.whitelist_mint_settings {
        let whitelist_token_account = get_ata_for_mint(&wl_mint_settings.mint, &payer);

        additional_accounts.push(AccountMeta {
            pubkey: whitelist_token_account,
            is_signer: false,
            is_writable: true,
        });

        if wl_mint_settings.mode == WhitelistMintMode::BurnEveryTime {
            let mut token_found = false;

            match program.rpc().get_account_data(&whitelist_token_account) {
                Ok(ata_data) => {
                    if !ata_data.is_empty() {
                        let account = Account::unpack_unchecked(&ata_data)?;

                        if account.amount > 0 {
                            additional_accounts.push(AccountMeta {
                                pubkey: wl_mint_settings.mint,
                                is_signer: false,
                                is_writable: true,
                            });

                            additional_accounts.push(AccountMeta {
                                pubkey: payer,
                                is_signer: true,
                                is_writable: false,
                            });

                            token_found = true;
                        }
                    }
                }
                Err(err) => return Err(anyhow!(err)),
            }

            if !token_found {
                return Err(anyhow!(ErrorCode::NoWhitelistToken));
            }
        }
    }

    if let Some(token_mint) = candy_machine_state.token_mint {
        let user_token_account_info = get_ata_for_mint(&token_mint, &payer);

        additional_accounts.push(AccountMeta {
            pubkey: user_token_account_info,
            is_signer: false,
            is_writable: true,
        });

        additional_accounts.push(AccountMeta {
            pubkey: payer,
            is_signer: true,
            is_writable: false,
        })
    }

    let metadata_pda = get_metadata_pda(&nft_mint.pubkey());
    let master_edition_pda = get_master_edition_pda(&nft_mint.pubkey());
    let (candy_machine_creator_pda, creator_bump) =
        get_candy_machine_creator_pda(&candy_machine_id);

    let mut builder = program
        .request()
        .instruction(create_mint_account_ix)
        .instruction(init_mint_ix)
        .instruction(create_assoc_account_ix)
        .instruction(mint_to_ix)
        .signer(&nft_mint)
        .accounts(nft_accounts::MintNFT {
            candy_machine: candy_machine_id,
            candy_machine_creator: candy_machine_creator_pda,
            payer,
            wallet,
            metadata: metadata_pda,
            mint: nft_mint.pubkey(),
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

    if !additional_accounts.is_empty() {
        for account in additional_accounts {
            builder = builder.accounts(account);
        }
    }

    let sig = builder.send()?;

    info!("Minted! TxId: {}", sig);

    Ok(sig)
}
