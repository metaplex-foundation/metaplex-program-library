use clap::{crate_description, crate_name, crate_version, App, Arg, SubCommand};
use solana_clap_utils::input_validators::{is_url, is_valid_signer};

pub const SHOW: &str = "show_hydra";

pub fn init_api<'a, 'b>() -> App<'a, 'b> {
    App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            Arg::with_name("keypair")
                .long("keypair")
                .value_name("KEYPAIR")
                .validator(is_valid_signer)
                .takes_value(true)
                .global(true)
                .help("Filepath or URL to a keypair"),
        )
        .arg(
            Arg::with_name("rpc")
                .long("json_rpc_url")
                .value_name("URL")
                .takes_value(true)
                .global(true)
                .validator(is_url)
                .help("JSON RPC URL for the cluster [default: devnet]"),
        )
        .subcommand(
            SubCommand::with_name(SHOW)
                .about("Show Hydra")
                .arg(
                    Arg::with_name("hydra_address")
                        .long("hydra_address")
                        .global(true)
                        .value_name("NAME")
                        .takes_value(true)
                        .help("The Hydra Address. Note this must be the parent address not the mint addresses"),
                )
        )
}
