use clap::{Parser, Subcommand};

use crate::constants::{DEFAULT_ASSETS, DEFAULT_CACHE, DEFAULT_CONFIG};

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    /// Log level: trace, debug, info, warn, error, off
    #[clap(short, long, global = true)]
    pub log_level: Option<String>,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Interactive process to create the config file
    CreateConfig {
        /// Path to the config file
        #[clap(short, long)]
        config: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the keypair file [default: solana config or "~/.config/solana/id.json"]
        #[clap(short, long)]
        keypair: Option<String>,

        /// Path to the directory with the assets
        #[clap(default_value = DEFAULT_ASSETS)]
        assets_dir: String,
    },
    /// Create a candy machine deployment from assets
    Launch {
        /// Path to the directory with the assets to upload
        #[clap(default_value = DEFAULT_ASSETS)]
        assets_dir: String,

        /// Path to the keypair file [default: solana config or "~/.config/solana/id.json"]
        #[clap(short, long)]
        keypair: Option<String>,

        /// Path to the config file
        #[clap(short, long, default_value = DEFAULT_CONFIG)]
        config: String,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Strict mode: validate against JSON metadata standard exactly
        #[clap(long)]
        strict: bool,

        /// Skip collection validate prompt
        #[clap(long)]
        skip_collection_prompt: bool,
    },
    /// Mint one NFT from candy machine
    Mint {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Amount of NFTs to be minted in bulk
        #[clap(short, long)]
        number: Option<u64>,

        /// Address of candy machine to mint from.
        #[clap(long)]
        candy_machine: Option<String>,
    },

    /// Update the candy machine config on-chain
    Update {
        /// Path to the config file, defaults to "config.json"
        #[clap(short, long, default_value = DEFAULT_CONFIG)]
        config: String,

        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Pubkey for the new authority
        #[clap(short, long)]
        new_authority: Option<String>,

        /// Address of candy machine to update.
        #[clap(long)]
        candy_machine: Option<String>,
    },

    /// Deploy cache items into candy machine config on-chain
    Deploy {
        /// Path to the config file, defaults to "config.json"
        #[clap(short, long, default_value = DEFAULT_CONFIG)]
        config: String,

        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,
    },

    /// Upload assets to storage and creates the cache config
    Upload {
        /// Path to the directory with the assets to upload
        #[clap(default_value = DEFAULT_ASSETS)]
        assets_dir: String,

        /// Path to the config file
        #[clap(short, long, default_value = DEFAULT_CONFIG)]
        config: String,

        /// Path to the keypair file [default: solana config or "~/.config/solana/id.json"]
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,
    },

    /// Withdraw funds from candy machine account closing it
    Withdraw {
        /// Address of candy machine to withdraw funds from.
        #[clap(long)]
        candy_machine: Option<String>,

        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// List available candy machines, no withdraw performed
        #[clap(long)]
        list: bool,
    },

    /// Validate JSON metadata files
    Validate {
        /// Assets directory to upload, defaults to "assets"
        #[clap(default_value = DEFAULT_ASSETS)]
        assets_dir: String,

        /// Strict mode: validate against JSON metadata standard exactly
        #[clap(long)]
        strict: bool,

        /// Skip collection prompt
        #[clap(long)]
        skip_collection_prompt: bool,
    },

    /// Verify uploaded data
    Verify {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,
    },

    /// Show the on-chain config of an existing candy machine
    Show {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Address of candy machine
        candy_machine: Option<String>,

        /// Display a list of unminted indices
        #[clap(long)]
        unminted: bool,
    },

    /// Interact with the bundlr network
    Bundlr {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        #[clap(subcommand)]
        action: BundlrAction,
    },

    /// Manage the collection on the candy machine
    Collection {
        #[clap(subcommand)]
        command: CollectionSubcommands,
    },
}

#[derive(Subcommand)]
pub enum CollectionSubcommands {
    /// Set the collection mint on the candy machine
    Set {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Address of candy machine to update.
        #[clap(long)]
        candy_machine: Option<String>,

        /// Address of collection mint to set the candy machine to.
        collection_mint: String,
    },

    /// Remove the collection from the candy machine
    Remove {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Address of candy machine to update.
        #[clap(long)]
        candy_machine: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum BundlrAction {
    /// Retrieve the balance on bundlr
    Balance,
    /// Withdraw funds from bundlr
    Withdraw,
}
