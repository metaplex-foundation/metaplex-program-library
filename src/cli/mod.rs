use clap::{AppSettings, Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about)]
#[clap(setting(AppSettings::SubcommandRequiredElseHelp))]
pub struct Cli {
    /// Log level: trace, debug, info, warn, error, off
    #[clap(short, long, global = true)]
    pub log_level: Option<String>,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Mint one NFT from candy machine
    MintOne {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = "cache.json")]
        cache: String,

        /// Amount of NFTs to be minted in bulk
        #[clap(short, long)]
        number: Option<u64>,
    },

    /// Upload assets to storage and then insert items into candy machine config
    Upload {
        /// Assets directory to upload, defaults to "assets"
        #[clap(default_value = "assets")]
        assets_dir: String,

        /// Arloader manifest file containing arweave links and asset names
        #[clap(default_value = "arloader-manifest.json")]
        arloader_manifest: String,

        /// Path to the config file, defaults to "config.json"
        #[clap(short, long, default_value = "config.json")]
        config: String,

        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = "cache.json")]
        cache: String,
    },

    UploadAssets {
        /// Assets directory to upload, defaults to "assets"
        #[clap(default_value = "assets")]
        assets_dir: String,

        /// Path to the config file, defaults to "config.json"
        #[clap(short, long, default_value = "config.json")]
        config: String,

        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = "cache.json")]
        cache: String,
    },

    /// Withdraw funds from candy machine account closing it.
    Withdraw {
        /// Address of candy machine to withdraw funds from.
        candy_machine: String,

        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,
    },

    /// Withdraw funds from candy machine account closing it.
    WithdrawAll {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,
    },

    /// Test command
    Test,

    /// Validate JSON metadata files
    Validate {
        /// Assets directory to upload, defaults to "assets"
        #[clap(default_value = "assets")]
        assets_dir: String,

        /// Strict mode: validate against JSON metadata standard exactly
        #[clap(long)]
        strict: bool,
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
        #[clap(long, default_value = "cache.json")]
        cache: String,
    },
}
