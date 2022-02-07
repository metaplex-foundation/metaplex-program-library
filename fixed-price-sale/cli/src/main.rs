mod cli_args;
mod error;
mod processor;
mod utils;

use chrono::prelude::*;
use clap::Parser;
use cli_args::{CliArgs, Commands};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signer::{keypair::read_keypair_file, Signer},
    transaction::Transaction,
};
use std::str::FromStr;

fn main() -> Result<(), error::Error> {
    let args = CliArgs::parse();

    let client = RpcClient::new(args.url);
    let payer_wallet = read_keypair_file(&args.payer_keypair)?;

    // Handle provided commands
    // Build transaction
    let tx_data: Option<(Transaction, Box<dyn processor::UiTransactionInfo>)> = match args.command {
        Commands::GetSellingResource { account } => {
            let selling_resource = processor::get_account_state::<
                mpl_fixed_price_sale::state::SellingResource,
            >(&client, &Pubkey::from_str(&account)?)?;

            println!("SellingResource::store - {}", selling_resource.store);
            println!("SellingResource::owner - {}", selling_resource.owner);
            println!("SellingResource::resource - {}", selling_resource.resource);
            println!("SellingResource::vault - {}", selling_resource.vault);
            println!(
                "SellingResource::vault_owner - {}",
                selling_resource.vault_owner
            );
            println!("SellingResource::supply - {}", selling_resource.supply);
            println!(
                "SellingResource::max_supply - {}",
                if let Some(x) = selling_resource.max_supply {
                    x.to_string()
                } else {
                    String::from("<unlimited>")
                }
            );
            println!("SellingResource::state - {:?}", selling_resource.state);

            None
        }
        Commands::GetStore { account } => {
            let store = processor::get_account_state::<mpl_fixed_price_sale::state::Store>(
                &client,
                &Pubkey::from_str(&account)?,
            )?;

            println!("Store::admin - {}", store.admin);
            println!("Store::name - {}", store.name);
            println!("Store::description - {}", store.description);

            None
        }
        Commands::GetMarket { account } => {
            let market = processor::get_account_state::<mpl_fixed_price_sale::state::Market>(
                &client,
                &Pubkey::from_str(&account)?,
            )?;

            let decimals = utils::get_mint(&client, &market.treasury_mint)?.decimals;

            println!("Market::store - {}", market.store);
            println!("Market::selling_resource - {}", market.selling_resource);
            println!("Market::treasury_mint - {}", market.treasury_mint);
            println!("Market::treasury_holder - {}", market.treasury_holder);
            println!("Market::treasury_owner - {}", market.treasury_owner);
            println!("Market::owner - {}", market.owner);
            println!("Market::name - {}", market.name);
            println!("Market::description - {}", market.description);
            println!("Market::mutable - {}", market.mutable);
            println!(
                "Market::price - {}",
                spl_token::amount_to_ui_amount(market.price, decimals)
            );
            println!(
                "Market::pieces_in_one_wallet - {}",
                if let Some(x) = market.pieces_in_one_wallet {
                    x.to_string()
                } else {
                    String::from("<unlimited>")
                }
            );
            println!("Market::start_date - {}", market.start_date);
            println!(
                "Market::end_date - {}",
                if let Some(x) = market.end_date {
                    x.to_string()
                } else {
                    String::from("<infinite>")
                }
            );
            println!("Market::state - {:?}", market.state);

            None
        }
        Commands::GetTradeHistory { account } => {
            let trade_history = processor::get_account_state::<
                mpl_fixed_price_sale::state::TradeHistory,
            >(&client, &Pubkey::from_str(&account)?)?;

            println!("TradeHistory::market - {}", trade_history.market);
            println!("TradeHistory::wallet - {}", trade_history.wallet);
            println!(
                "TradeHistory::already_bought - {}",
                trade_history.already_bought
            );

            None
        }
        Commands::CreateStore {
            admin_keypair,
            name,
            description,
        } => {
            let admin_keypair = if let Some(keypair) = admin_keypair {
                read_keypair_file(keypair)?
            } else {
                utils::clone_keypair(&payer_wallet)
            };

            Some(processor::create_store(
                &client,
                &payer_wallet,
                &admin_keypair,
                &name,
                &description,
            )?)
        }
        Commands::InitSellingResource {
            store,
            admin_keypair,
            selling_resource_owner,
            resource_mint,
            resource_token,
            max_supply,
        } => {
            let admin_keypair = if let Some(keypair) = admin_keypair {
                read_keypair_file(keypair)?
            } else {
                utils::clone_keypair(&payer_wallet)
            };

            let selling_resource_owner = if let Some(owner) = selling_resource_owner {
                Pubkey::from_str(&owner)?
            } else {
                payer_wallet.pubkey()
            };

            Some(processor::init_selling_resource(
                &client,
                &payer_wallet,
                &Pubkey::from_str(&store)?,
                &admin_keypair,
                &selling_resource_owner,
                &Pubkey::from_str(&resource_mint)?,
                &Pubkey::from_str(&resource_token)?,
                max_supply,
            )?)
        }
        Commands::CreateMarket {
            selling_resource_owner_keypair,
            selling_resource,
            mint,
            name,
            description,
            mutable,
            price,
            pieces_in_one_wallet,
            start_date,
            end_date,
        } => {
            let selling_resource_owner = if let Some(owner) = selling_resource_owner_keypair {
                read_keypair_file(&owner)?
            } else {
                utils::clone_keypair(&payer_wallet)
            };

            let mint = if let Some(mint) = mint {
                Some(Pubkey::from_str(&mint)?)
            } else {
                None
            };

            let mint = if let Some(mint) = mint {
                mint
            } else {
                spl_token::native_mint::id()
            };

            let start_date = if let Some(start_date) = start_date {
                start_date as u64
            } else {
                Utc::now().timestamp() as u64
            };

            let decimals = utils::get_mint(&client, &mint)?.decimals;

            Some(processor::create_market(
                &client,
                &payer_wallet,
                &selling_resource_owner,
                &Pubkey::from_str(&selling_resource)?,
                &mint,
                &name,
                &description,
                mutable,
                spl_token::ui_amount_to_amount(price, decimals),
                pieces_in_one_wallet,
                start_date,
                end_date,
            )?)
        }
        Commands::Buy {
            market,
            user_token_account,
            user_wallet_keypair,
        } => {
            let user_wallet = if let Some(keypair) = user_wallet_keypair {
                read_keypair_file(keypair)?
            } else {
                utils::clone_keypair(&payer_wallet)
            };

            Some(processor::buy(
                &client,
                &payer_wallet,
                &Pubkey::from_str(&market)?,
                &Pubkey::from_str(&user_token_account)?,
                &user_wallet,
            )?)
        }
    };

    // Send builded transaction
    if let Some((tx, ui_info)) = tx_data {
        client.send_and_confirm_transaction(&tx)?;
        ui_info.print();
    }

    Ok(())
}
