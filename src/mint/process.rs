use std::{str::FromStr, sync::Arc};

use anchor_client::solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    system_instruction, system_program, sysvar,
};
use anyhow::Result;
use borsh::BorshDeserialize;
use console::style;
use mpl_candy_machine_core::{
    accounts as nft_accounts, instruction as nft_instruction, CandyMachine,
};
use mpl_token_metadata::{pda::find_collection_authority_account, state::Metadata};
use solana_client::rpc_response::Response;
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::{
    instruction::{initialize_mint, mint_to},
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
            receiver_pubkey,
        )
        .await
        {
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

        let mut tasks = Vec::new();
        let semaphore = Arc::new(Semaphore::new(10));
        let config = Arc::new(sugar_config);

        for _i in 0..number {
            let config = config.clone();
            let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();
            let candy_machine_state = candy_machine_state.clone();
            let pb = pb.clone();

            // Start tasks
            tasks.push(tokio::spawn(async move {
                let _permit = permit;
                let res = mint(config, candy_pubkey, candy_machine_state, receiver_pubkey).await;
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
    receiver: Pubkey,
) -> Result<Signature> {
    let client = setup_client(&config)?;
    let program = client.program(CANDY_MACHINE_ID);
    let payer = program.payer();

    if candy_machine_state.mint_authority != payer {
        return Err(anyhow!("Payer is not the mint authority."));
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

    let collection_mint = candy_machine_state.collection_mint;

    let (authority_pda, _) = find_candy_machine_creator_pda(&candy_machine_id);
    let collection_authority_record =
        find_collection_authority_account(&collection_mint, &authority_pda).0;
    let collection_metadata = find_metadata_pda(&collection_mint);

    let data = program.rpc().get_account_data(&collection_metadata)?;
    let metadata: Metadata = BorshDeserialize::deserialize(&mut data.as_slice())?;

    let metadata_pda = find_metadata_pda(&nft_mint.pubkey());
    let master_edition_pda = find_master_edition_pda(&nft_mint.pubkey());

    let mint_ix = program
        .request()
        .accounts(nft_accounts::Mint {
            candy_machine: candy_machine_id,
            authority_pda,
            payer,
            mint_authority: payer,
            nft_metadata: metadata_pda,
            nft_mint: nft_mint.pubkey(),
            nft_master_edition: master_edition_pda,
            nft_mint_authority: payer,
            collection_mint: candy_machine_state.collection_mint,
            collection_metadata: find_metadata_pda(&candy_machine_state.collection_mint),
            collection_master_edition: find_master_edition_pda(
                &candy_machine_state.collection_mint,
            ),
            collection_authority_record,
            collection_update_authority: metadata.update_authority,
            token_metadata_program: metaplex_program_id,
            token_program: TOKEN_PROGRAM_ID,
            system_program: system_program::id(),
            recent_slothashes: sysvar::slot_hashes::ID,
        })
        .args(nft_instruction::Mint {});

    let mint_ix = mint_ix.instructions()?;

    let builder = program
        .request()
        .instruction(create_mint_account_ix)
        .instruction(init_mint_ix)
        .instruction(create_assoc_account_ix)
        .instruction(mint_to_ix)
        .instruction(mint_ix[0].clone())
        .signer(&nft_mint);

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
