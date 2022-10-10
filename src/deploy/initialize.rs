use anchor_client::solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    system_instruction, system_program,
};
use anyhow::Result;
use mpl_candy_machine_core::{
    accounts as nft_accounts, instruction as nft_instruction, CandyMachineData, ConfigLineSettings,
    Creator as CandyCreator,
};
use mpl_token_metadata::pda::find_collection_authority_account;
pub use mpl_token_metadata::state::{
    MAX_CREATOR_LIMIT, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH,
};
use solana_program::native_token::LAMPORTS_PER_SOL;

use crate::{
    common::*,
    config::data::*,
    deploy::errors::*,
    pdas::{find_candy_machine_creator_pda, find_master_edition_pda, find_metadata_pda},
};

/// Create the candy machine data struct.
pub fn create_candy_machine_data(
    _client: &Client,
    config: &ConfigData,
    cache: &Cache,
) -> Result<CandyMachineData> {
    let mut creators: Vec<CandyCreator> = Vec::new();
    let mut share = 0u32;

    for creator in &config.creators {
        let c = creator.to_candy_format()?;
        share += c.percentage_share as u32;

        creators.push(c);
    }

    if creators.is_empty() || creators.len() > (MAX_CREATOR_LIMIT - 1) {
        return Err(anyhow!(
            "The number of creators must be between 1 and {}.",
            MAX_CREATOR_LIMIT - 1,
        ));
    }

    if share != 100 {
        return Err(anyhow!(
            "Creator(s) share must add up to 100, current total {}.",
            share,
        ));
    }

    // CMv3 allows the specification of a common prefix for both name and uri
    // therefore we need to determine the largest common prefix and the len of
    // the remaining parts of the name and uri

    // (shortest, largest, len largest) name
    let mut name_pair = [String::new(), String::new(), String::new()];
    // (shortest, largest, len largest) uri
    let mut uri_pair = [String::new(), String::new(), String::new()];
    // compares a value against a pair of (shorter, larger)
    let compare_pair = |value: &String, pair: &mut [String; 3]| {
        // lexicographic smaller
        if pair[0].is_empty() || value < &pair[0] {
            pair[0] = value.to_string();
        }
        // lexicographic larger
        if value > &pair[1] {
            pair[1] = value.to_string();
        }
        // lengthwise larger
        if value.len() > pair[2].len() {
            pair[2] = value.to_string();
        }
    };
    let common_prefix = |value1: &str, value2: &str| {
        let bytes1 = value1.as_bytes();
        let bytes2 = value2.as_bytes();
        let mut index = 0;

        while (index < bytes1.len() && index < bytes2.len()) && bytes1[index] == bytes2[index] {
            index += 1;
        }

        value1[..index].to_string()
    };

    for (index, item) in cache.items.iter() {
        if i64::from_str(index)? > -1 {
            compare_pair(&item.name, &mut name_pair);
            compare_pair(&item.metadata_link, &mut uri_pair);
        }
    }

    let name_prefix = common_prefix(&name_pair[0], &name_pair[1]);
    let uri_prefix = common_prefix(&uri_pair[0], &uri_pair[1]);

    let config_line_settings = ConfigLineSettings {
        name_length: (name_pair[2].len() - name_prefix.len()) as u32,
        prefix_name: name_prefix,
        uri_length: (uri_pair[2].len() - uri_prefix.len()) as u32,
        prefix_uri: uri_prefix,
        is_sequential: config.is_sequential,
    };

    let hidden_settings = config.hidden_settings.as_ref().map(|s| s.to_candy_format());

    let data = CandyMachineData {
        items_available: config.size,
        symbol: config.symbol.clone(),
        seller_fee_basis_points: config.royalties,
        max_supply: 0,
        is_mutable: config.is_mutable,
        creators,
        config_line_settings: Some(config_line_settings),
        hidden_settings,
    };

    Ok(data)
}

/// Send the `initialize_candy_machine` instruction to the candy machine program.
pub fn initialize_candy_machine(
    _config_data: &ConfigData,

    candy_account: &Keypair,
    candy_machine_data: CandyMachineData,
    collection_mint: Pubkey,
    collection_update_authority: Pubkey,
    program: Program,
) -> Result<Signature> {
    let payer = program.payer();
    let candy_account_size = candy_machine_data.get_space_for_candy()?;

    info!(
        "Initializing candy machine with account size of: {} and address of: {}",
        candy_account_size,
        candy_account.pubkey().to_string()
    );

    let lamports = program
        .rpc()
        .get_minimum_balance_for_rent_exemption(candy_account_size)?;

    let balance = program.rpc().get_account(&payer)?.lamports;

    if lamports > balance {
        return Err(DeployError::BalanceTooLow(
            format!("{:.3}", (balance as f64 / LAMPORTS_PER_SOL as f64)),
            format!("{:.3}", (lamports as f64 / LAMPORTS_PER_SOL as f64)),
        )
        .into());
    }

    // required PDAs

    let (authority_pda, _) = find_candy_machine_creator_pda(&candy_account.pubkey());
    let collection_authority_record =
        find_collection_authority_account(&collection_mint, &authority_pda).0;
    let collection_metadata = find_metadata_pda(&collection_mint);
    let collection_master_edition = find_master_edition_pda(&collection_mint);

    let tx = program
        .request()
        .instruction(system_instruction::create_account(
            &payer,
            &candy_account.pubkey(),
            lamports,
            candy_account_size as u64,
            &program.id(),
        ))
        .signer(candy_account)
        .accounts(nft_accounts::Initialize {
            candy_machine: candy_account.pubkey(),
            authority: payer,
            authority_pda,
            payer,
            collection_metadata,
            collection_mint,
            collection_master_edition,
            collection_authority_record,
            collection_update_authority,
            token_metadata_program: mpl_token_metadata::ID,
            system_program: system_program::id(),
        })
        .args(nft_instruction::Initialize {
            data: candy_machine_data,
        });

    let sig = tx.send()?;

    Ok(sig)
}
