use std::str::FromStr;

use anchor_client::solana_sdk::{pubkey::Pubkey, system_program, sysvar};
use anyhow::Result;
use console::style;
use mpl_candy_machine::{accounts as nft_accounts, instruction as nft_instruction, CandyError};
use mpl_token_metadata::{
    error::MetadataError,
    pda::find_collection_authority_account,
    state::{MasterEditionV2, Metadata},
};

use crate::{
    cache::load_cache,
    candy_machine::{CANDY_MACHINE_ID, *},
    common::*,
    pdas::*,
    utils::spinner_with_style,
};

pub struct SetCollectionArgs {
    pub collection_mint: String,
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub candy_machine: Option<String>,
}

pub fn process_set_collection(args: SetCollectionArgs) -> Result<()> {
    let sugar_config = sugar_setup(args.keypair, args.rpc_url)?;
    let client = setup_client(&sugar_config)?;
    let program = client.program(CANDY_MACHINE_ID);
    let mut cache = Cache::new();

    // The candy machine id specified takes precedence over the one from the cache.
    let candy_machine_id = match args.candy_machine {
        Some(ref candy_machine_id) => candy_machine_id,
        None => {
            cache = load_cache(&args.cache, false)?;
            &cache.program.candy_machine
        }
    };

    let collection_mint_pubkey = match Pubkey::from_str(&args.collection_mint) {
        Ok(candy_pubkey) => candy_pubkey,
        Err(_) => {
            let error = anyhow!(
                "Failed to parse collection mint id: {}",
                args.collection_mint
            );
            error!("{:?}", error);
            return Err(error);
        }
    };

    let candy_pubkey = match Pubkey::from_str(candy_machine_id) {
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

    let candy_machine_state =
        get_candy_machine_state(&sugar_config, &Pubkey::from_str(candy_machine_id)?)?;

    let collection_metadata_info = get_metadata_pda(&collection_mint_pubkey, &program)?;

    let collection_edition_info = get_master_edition_pda(&collection_mint_pubkey, &program)?;

    pb.finish_with_message("Done");

    println!(
        "\n{} {}Setting collection mint for candy machine",
        style("[2/2]").bold().dim(),
        COLLECTION_EMOJI
    );

    let pb = spinner_with_style();
    pb.set_message("Sending set collection transaction...");

    let set_signature = set_collection(
        &program,
        &candy_pubkey,
        &candy_machine_state,
        &collection_mint_pubkey,
        &collection_metadata_info,
        &collection_edition_info,
    )?;

    // If a candy machine id wasn't manually specified we are operating on the candy machine in the cache
    // and so need to update the cache file.
    if args.candy_machine.is_none() {
        cache.items.shift_remove("-1");
        cache.program.collection_mint = collection_mint_pubkey.to_string();
        cache.sync_file()?;
    }

    pb.finish_with_message(format!(
        "{} {}",
        style("Set collection signature:").bold(),
        set_signature
    ));

    Ok(())
}

pub fn set_collection(
    program: &Program,
    candy_pubkey: &Pubkey,
    candy_machine_state: &CandyMachine,
    collection_mint_pubkey: &Pubkey,
    collection_metadata_info: &PdaInfo<Metadata>,
    collection_edition_info: &PdaInfo<MasterEditionV2>,
) -> Result<Signature> {
    let payer = program.payer();

    let collection_pda_pubkey = find_collection_pda(candy_pubkey).0;
    let (collection_metadata_pubkey, collection_metadata) = collection_metadata_info;
    let (collection_edition_pubkey, collection_edition) = collection_edition_info;

    let collection_authority_record =
        find_collection_authority_account(collection_mint_pubkey, &collection_pda_pubkey).0;

    if !candy_machine_state.data.retain_authority {
        return Err(anyhow!(CandyError::CandyCollectionRequiresRetainAuthority));
    }

    if collection_metadata.update_authority != payer {
        return Err(anyhow!(CustomCandyError::AuthorityMismatch(
            collection_metadata.update_authority.to_string(),
            payer.to_string()
        )));
    }

    if collection_edition.max_supply != Some(0) {
        return Err(anyhow!(MetadataError::CollectionMustBeAUniqueMasterEdition));
    }

    if candy_machine_state.items_redeemed > 0 {
        return Err(anyhow!(
            "You can't modify the Candy Machine collection after items have been minted."
        ));
    }

    let builder = program
        .request()
        .accounts(nft_accounts::SetCollection {
            candy_machine: *candy_pubkey,
            authority: payer,
            collection_pda: collection_pda_pubkey,
            payer,
            system_program: system_program::id(),
            rent: sysvar::rent::ID,
            metadata: *collection_metadata_pubkey,
            mint: *collection_mint_pubkey,
            edition: *collection_edition_pubkey,
            collection_authority_record,
            token_metadata_program: mpl_token_metadata::ID,
        })
        .args(nft_instruction::SetCollection);

    let sig = builder.send()?;

    Ok(sig)
}
