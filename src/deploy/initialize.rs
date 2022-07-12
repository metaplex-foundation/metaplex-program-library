use anchor_client::solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    system_instruction, system_program, sysvar,
};
use anchor_lang::prelude::AccountMeta;
use anyhow::Result;
use mpl_candy_machine::{
    accounts as nft_accounts, instruction as nft_instruction, CandyMachineData,
    Creator as CandyCreator,
};
pub use mpl_token_metadata::state::{
    MAX_CREATOR_LIMIT, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH,
};
use solana_program::native_token::LAMPORTS_PER_SOL;

use crate::{candy_machine::parse_config_price, common::*, config::data::*, deploy::errors::*};

/// Create the candy machine data struct.
pub fn create_candy_machine_data(
    client: &Client,
    config: &ConfigData,
    uuid: String,
) -> Result<CandyMachineData> {
    let go_live_date: Option<i64> = go_live_date_as_timestamp(&config.go_live_date)?;

    let end_settings = config.end_settings.as_ref().map(|s| s.to_candy_format());

    let whitelist_mint_settings = config
        .whitelist_mint_settings
        .as_ref()
        .map(|s| s.to_candy_format());

    let hidden_settings = config.hidden_settings.as_ref().map(|s| s.to_candy_format());

    let gatekeeper = config
        .gatekeeper
        .as_ref()
        .map(|gatekeeper| gatekeeper.to_candy_format());

    let mut creators: Vec<CandyCreator> = Vec::new();
    let mut share = 0u32;

    for creator in &config.creators {
        let c = creator.to_candy_format()?;
        share += c.share as u32;

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

    let price = parse_config_price(client, config)?;

    let data = CandyMachineData {
        uuid,
        price,
        symbol: config.symbol.clone(),
        seller_fee_basis_points: config.seller_fee_basis_points,
        max_supply: 0,
        is_mutable: config.is_mutable,
        retain_authority: config.retain_authority,
        go_live_date,
        end_settings,
        creators,
        whitelist_mint_settings,
        hidden_settings,
        items_available: config.number,
        gatekeeper,
    };

    Ok(data)
}

/// Send the `initialize_candy_machine` instruction to the candy machine program.
pub fn initialize_candy_machine(
    config_data: &ConfigData,
    candy_account: &Keypair,
    candy_machine_data: CandyMachineData,
    treasury_wallet: Pubkey,
    program: Program,
) -> Result<Signature> {
    let payer = program.payer();
    let items_available = candy_machine_data.items_available;

    let candy_account_size = if candy_machine_data.hidden_settings.is_some() {
        CONFIG_ARRAY_START
    } else {
        CONFIG_ARRAY_START
            + 4
            + items_available as usize * CONFIG_LINE_SIZE
            + 8
            + 2 * (items_available as usize / 8 + 1)
    };

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

    let mut tx = program
        .request()
        .instruction(system_instruction::create_account(
            &payer,
            &candy_account.pubkey(),
            lamports,
            candy_account_size as u64,
            &program.id(),
        ))
        .signer(candy_account)
        .accounts(nft_accounts::InitializeCandyMachine {
            candy_machine: candy_account.pubkey(),
            wallet: treasury_wallet,
            authority: payer,
            payer,
            system_program: system_program::id(),
            rent: sysvar::rent::ID,
        })
        .args(nft_instruction::InitializeCandyMachine {
            data: candy_machine_data,
        });

    if let Some(token) = config_data.spl_token {
        tx = tx.accounts(AccountMeta {
            pubkey: token,
            is_signer: false,
            is_writable: false,
        });
    }

    let sig = tx.send()?;

    Ok(sig)
}
