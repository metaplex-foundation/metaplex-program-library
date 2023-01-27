use std::{str::FromStr, sync::Arc};

use anchor_client::solana_sdk::{
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    system_instruction, system_program, sysvar,
};
use anchor_lang::{prelude::AccountMeta, ToAccountMetas};
use anyhow::Result;
use chrono::Utc;
use console::style;
use mpl_candy_machine::{
    accounts as nft_accounts, instruction as nft_instruction, CandyError, CandyMachine,
    CollectionPDA, EndSettingType, WhitelistMintMode,
};
use mpl_token_metadata::pda::find_collection_authority_account;
use solana_client::rpc_response::Response;
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::{
    instruction::{initialize_mint, mint_to},
    state::Account,
    ID as TOKEN_PROGRAM_ID,
};
use tokio::sync::Semaphore;

use crate::{
    cache::load_cache,
    candy_machine::{CANDY_MACHINE_ID, *},
    common::*,
    config::{Cluster, SugarConfig},
    pdas::*,
    utils::*,
};

pub struct MintArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub number: Option<u64>,
    pub receiver: Option<String>,
    pub candy_machine: Option<String>,
}

pub async fn process_mint(args: MintArgs) -> Result<()> {
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
        "{} {}Loading candy machine",
        style("[1/2]").bold().dim(),
        LOOKING_GLASS_EMOJI
    );
    println!("{} {}", style("Candy machine ID:").bold(), candy_machine_id);

    let pb = spinner_with_style();
    pb.set_message("Connecting...");

    let candy_machine_state = Arc::new(get_candy_machine_state(&sugar_config, &candy_pubkey)?);

    let collection_pda_info =
        Arc::new(get_collection_pda(&candy_pubkey, &client.program(CANDY_MACHINE_ID)).ok());

    pb.finish_with_message("Done");

    println!(
        "\n{} {}Minting from candy machine",
        style("[2/2]").bold().dim(),
        CANDY_EMOJI
    );

    let receiver_pubkey = match args.receiver {
        Some(receiver_id) => Pubkey::from_str(&receiver_id)
            .map_err(|_| anyhow!("Failed to parse receiver pubkey: {}", receiver_id))?,
        None => sugar_config.keypair.pubkey(),
    };
    println!("\nMinting to {}", &receiver_pubkey);

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
        let config = Arc::new(sugar_config);

        let result = match mint(
            Arc::clone(&config),
            candy_pubkey,
            Arc::clone(&candy_machine_state),
            Arc::clone(&collection_pda_info),
            receiver_pubkey,
        )
        .await
        {
            Ok(signature) => {
                println!("Signature: {signature}");
                format!("{}", style("Mint success").bold())
            }
            Err(err) => {
                pb.abandon_with_message(format!("{}", style("Mint failed ").red().bold()));
                error!("{:?}", err);
                return Err(err);
            }
        };

        pb.finish_with_message(result);
    } else {
        let pb = progress_bar_with_style(number);

        let mut tasks = Vec::new();
        let semaphore = Arc::new(Semaphore::new(10));
        let config = Arc::new(sugar_config);

        for _i in 0..number {
            let config = config.clone();
            let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();
            let candy_machine_state = candy_machine_state.clone();
            let collection_pda_info = collection_pda_info.clone();
            let pb = pb.clone();

            // Start tasks
            tasks.push(tokio::spawn(async move {
                let _permit = permit;
                let res = mint(
                    config,
                    candy_pubkey,
                    candy_machine_state,
                    collection_pda_info,
                    receiver_pubkey,
                )
                .await;
                pb.inc(1);
                res
            }));
        }

        let mut error_count = 0;

        // Resolve tasks
        for task in tasks {
            let res = task.await.unwrap();
            if let Err(e) = res {
                error_count += 1;
                error!("{:?}, continuing. . .", e);
            }
        }

        if error_count > 0 {
            pb.abandon_with_message(format!(
                "{} {} items failed.",
                style("Some of the items failed to mint.").red().bold(),
                error_count
            ));
            return Err(anyhow!(
                "{} {}/{} {}",
                style("Minted").red().bold(),
                number - error_count,
                number,
                style("of the items").red().bold()
            ));
        }
        pb.finish();
    }

    Ok(())
}

pub async fn mint(
    config: Arc<SugarConfig>,
    candy_machine_id: Pubkey,
    candy_machine_state: Arc<CandyMachine>,
    collection_pda_info: Arc<Option<PdaInfo<CollectionPDA>>>,
    receiver: Pubkey,
) -> Result<Signature> {
    let client = setup_client(&config)?;
    let program = client.program(CANDY_MACHINE_ID);
    let payer = program.payer();
    let wallet = candy_machine_state.wallet;
    let authority = candy_machine_state.authority;

    let candy_machine_data = &candy_machine_state.data;

    if let Some(_gatekeeper) = &candy_machine_data.gatekeeper {
        return Err(anyhow!(
            "Command-line mint disabled (gatekeeper settings in use)"
        ));
    } else if candy_machine_state.items_redeemed >= candy_machine_data.items_available {
        return Err(anyhow!(CandyError::CandyMachineEmpty));
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
                return Err(anyhow!(CandyError::CandyMachineNotLive));
            }
        }

        if !mint_enabled {
            // no whitelist mint settings (or no presale) and we are earlier than
            // go live date
            return Err(anyhow!(CandyError::CandyMachineNotLive));
        }

        if let Some(end_settings) = &candy_machine_data.end_settings {
            match end_settings.end_setting_type {
                EndSettingType::Date => {
                    if (end_settings.number as i64) < mint_date {
                        return Err(anyhow!(CandyError::CandyMachineNotLive));
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
    let assoc = get_associated_token_address(&receiver, &nft_mint.pubkey());

    // Create associated account instruction
    let create_assoc_account_ix =
        create_associated_token_account(&payer, &receiver, &nft_mint.pubkey());

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
        let whitelist_token_account = get_associated_token_address(&payer, &wl_mint_settings.mint);

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
                Err(err) => {
                    error!("Invalid whitelist token account: {}", err);
                    return Err(anyhow!(
                        "Uninitialized whitelist token account: {whitelist_token_account}.
                         Check that you provided a valid SPL token mint for the whitelist."
                    ));
                }
            }

            if !token_found {
                return Err(anyhow!(CandyError::NoWhitelistToken));
            }
        }
    }

    if let Some(token_mint) = candy_machine_state.token_mint {
        let user_token_account_info = get_associated_token_address(&payer, &token_mint);

        additional_accounts.push(AccountMeta {
            pubkey: user_token_account_info,
            is_signer: false,
            is_writable: true,
        });

        additional_accounts.push(AccountMeta {
            pubkey: payer,
            is_signer: true,
            is_writable: false,
        });
    }

    // If the freeze PDA exists, add the additional account.
    let (pda, _) = find_freeze_pda(&candy_machine_id);
    let freeze_pda_opt = get_freeze_pda_account(&config, &candy_machine_id).ok();
    if freeze_pda_opt.is_some() {
        additional_accounts.push(AccountMeta {
            pubkey: pda,
            is_signer: false,
            is_writable: true,
        });
        let nft_token_account = get_associated_token_address(&payer, &nft_mint.pubkey());

        additional_accounts.push(AccountMeta {
            pubkey: nft_token_account,
            is_signer: false,
            is_writable: false,
        });

        // Add freeze ata if SPL token mint is enabled.
        if let Some(token_mint) = candy_machine_state.token_mint {
            let freeze_ata = get_associated_token_address(&pda, &token_mint);
            additional_accounts.push(AccountMeta {
                pubkey: freeze_ata,
                is_signer: false,
                is_writable: true,
            });
        }
    }

    let metadata_pda = find_metadata_pda(&nft_mint.pubkey());
    let master_edition_pda = find_master_edition_pda(&nft_mint.pubkey());
    let (candy_machine_creator_pda, creator_bump) =
        find_candy_machine_creator_pda(&candy_machine_id);

    let mut mint_ix = program
        .request()
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

    // Add additional accounts directly to the mint instruction otherwise it won't work.
    if !additional_accounts.is_empty() {
        mint_ix = mint_ix.accounts(additional_accounts);
    }
    let mint_ix = mint_ix.instructions()?;

    let mut builder = program
        .request()
        .instruction(create_mint_account_ix)
        .instruction(init_mint_ix)
        .instruction(create_assoc_account_ix)
        .instruction(mint_to_ix)
        .instruction(mint_ix[0].clone())
        .signer(&nft_mint);

    if let Some((collection_pda_pubkey, collection_pda)) = collection_pda_info.as_ref() {
        let collection_authority_record =
            find_collection_authority_account(&collection_pda.mint, collection_pda_pubkey).0;

        let mut accounts = nft_accounts::SetCollectionDuringMint {
            candy_machine: candy_machine_id,
            metadata: metadata_pda,
            payer,
            collection_pda: *collection_pda_pubkey,
            token_metadata_program: mpl_token_metadata::ID,
            instructions: sysvar::instructions::ID,
            collection_mint: collection_pda.mint,
            collection_metadata: find_metadata_pda(&collection_pda.mint),
            collection_master_edition: find_master_edition_pda(&collection_pda.mint),
            authority,
            collection_authority_record,
        }
        .to_account_metas(None);

        // Make collection metadata account mutable for sized collections.
        accounts[7].is_writable = true;

        builder = builder
            .accounts(accounts)
            .args(nft_instruction::SetCollectionDuringMint {});
    }

    let sig = builder.send()?;

    if let Err(_) | Ok(Response { value: None, .. }) = program
        .rpc()
        .get_account_with_commitment(&metadata_pda, CommitmentConfig::processed())
    {
        let cluster_param = match get_cluster(program.rpc()).unwrap_or(Cluster::Mainnet) {
            Cluster::Devnet => "?devnet",
            _ => "",
        };
        return Err(anyhow!(
            "Minting most likely failed with a bot tax. Check the transaction link for more details: https://explorer.solana.com/tx/{}{}",
            sig.to_string(),
            cluster_param,
        ));
    }

    info!("Minted! TxId: {}", sig);

    Ok(sig)
}
