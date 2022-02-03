//! Module define CLI structure.

use clap::{Parser, Subcommand};

/// CLI arguments.
#[derive(Parser, Debug)]
#[clap(name = "mpl-fixed-price-sale-cli")]
#[clap(about = "CLI utility for mpl-fixed-price-sale program")]
#[clap(version, author)]
pub struct CliArgs {
    /// RPC endpoint.
    #[clap(short, long, default_value_t = String::from("https://api.mainnet-beta.solana.com"), value_name = "URL")]
    pub url: String,

    /// Path to transaction payer keypair file.
    #[clap(short, long, default_value_t = String::from("~/.config/solana/id.json"), value_name = "FILE")]
    pub payer_keypair: String,

    #[clap(subcommand)]
    pub command: Commands,
}

/// CLI sub-commands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Obtain `SellingResource` account from `mpl_fixed_price_sale` program.
    GetSellingResource {
        #[clap(short, value_name = "STRING")]
        account: String,
    },
    /// Obtain `Store` account from `mpl_fixed_price_sale` program.
    GetStore {
        #[clap(short, value_name = "STRING")]
        account: String,
    },
    /// Obtain `Market` account from `mpl_fixed_price_sale` program.
    GetMarket {
        #[clap(short, value_name = "STRING")]
        account: String,
    },
    /// Obtain `TradeHistory` account from `mpl_fixed_price_sale` program.
    GetTradeHistory {
        #[clap(short, value_name = "STRING")]
        account: String,
    },
    /// Perform `CreateStore` instruction of `mpl_fixed_price_sale` program.
    CreateStore {
        #[clap(long, value_name = "FILE")]
        admin_keypair: Option<String>,

        #[clap(long, value_name = "STRING")]
        name: String,

        #[clap(long, value_name = "STRING")]
        description: String,
    },
    /// Perform `Buy` instruction of `mpl_fixed_price_sale` program.
    Buy {
        #[clap(long, value_name = "PUBKEY")]
        market: String,

        #[clap(long, value_name = "PUBKEY")]
        user_token_account: String,

        #[clap(long, value_name = "FILE")]
        user_wallet_keypair: Option<String>,
    },
    /// Perform `InitSellingResource` instruction of `mpl_fixed_price_sale` program.
    InitSellingResource {
        #[clap(long, value_name = "PUBKEY")]
        store: String,

        #[clap(long, value_name = "FILE")]
        admin_keypair: Option<String>,

        #[clap(long, value_name = "PUBKEY")]
        selling_resource_owner: Option<String>,

        #[clap(long, value_name = "PUBKEY")]
        resource_mint: String,

        #[clap(long, value_name = "PUBKEY")]
        resource_token: String,

        #[clap(long, value_name = "U64")]
        max_supply: Option<u64>,
    },
    /// Perform `CreateMarket` instruction of `mpl_fixed_price_sale` program.
    CreateMarket {
        #[clap(long, value_name = "FILE")]
        selling_resource_owner_keypair: Option<String>,

        #[clap(long, value_name = "PUBKEY")]
        selling_resource: String,

        #[clap(long, value_name = "PUBKEY")]
        mint: Option<String>,

        #[clap(long, value_name = "STRING")]
        name: String,

        #[clap(long, value_name = "STRING")]
        description: String,

        #[clap(long, value_name = "BOOL")]
        mutable: bool,

        #[clap(long, value_name = "F64")]
        price: f64,

        #[clap(long, value_name = "U64")]
        pieces_in_one_wallet: Option<u64>,

        #[clap(long, value_name = "TIMESTAMP")]
        start_date: Option<u64>,

        #[clap(long, value_name = "TIMESTAMP")]
        end_date: Option<u64>,
    },
}
