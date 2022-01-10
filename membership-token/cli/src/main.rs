mod cli_args;
mod error;
mod processor;
mod utils;

use clap::Parser;
use cli_args::{CliArgs, Commands};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signer::keypair::read_keypair_file, transaction::Transaction};
use std::str::FromStr;

fn main() -> Result<(), error::Error> {
    let args = CliArgs::parse();

    let client = RpcClient::new(args.url);
    let _payer_wallet = read_keypair_file(&args.payer_keypair)?;

    // Handle provided commands
    // Build transaction
    let tx_data: Option<(Transaction, Box<dyn ToString>)> = match args.command {
        Commands::GetSellingResource { account } => {
            let selling_resource = processor::get_account_state::<
                mpl_membership_token::state::SellingResource,
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
            let store = processor::get_account_state::<mpl_membership_token::state::Store>(
                &client,
                &Pubkey::from_str(&account)?,
            )?;

            println!("Store::admin - {}", store.admin);
            println!("Store::name - {}", store.name);
            println!("Store::description - {}", store.description);

            None
        }
        Commands::GetMarket { account } => {
            let market = processor::get_account_state::<mpl_membership_token::state::Market>(
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
                mpl_membership_token::state::TradeHistory,
            >(&client, &Pubkey::from_str(&account)?)?;

            println!("TradeHistory::market - {}", trade_history.market);
            println!("TradeHistory::wallet - {}", trade_history.wallet);
            println!(
                "TradeHistory::already_bought - {}",
                trade_history.already_bought
            );

            None
        }
        _ => None,
    };

    // Process builded transaction
    if let Some((tx, data)) = tx_data {
        client.send_and_confirm_transaction(&tx)?;
        println!("{}", data.to_string());
    }

    Ok(())
}
