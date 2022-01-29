use crate::common::*;

use crate::verify::VerifyError;

pub struct VerifyArgs {
    pub logger: Logger,
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
}

#[derive(Debug)]
pub struct OnChainItem {
    pub name: String,
    pub uri: String,
}

pub fn process_verify(args: VerifyArgs) -> Result<()> {
    let sugar_config = match sugar_setup(args.logger, args.keypair, args.rpc_url) {
        Ok(sugar_config) => sugar_config,
        Err(err) => {
            return Err(SetupError::SugarSetupError(err.to_string()).into());
        }
    };

    let cache_file_path = Path::new(&args.cache);

    if !cache_file_path.exists() {
        let cache_file_string = path_to_string(&cache_file_path)?;
        return Err(CacheError::CacheFileNotFound(cache_file_string).into());
    }

    info!(sugar_config.logger, "Cache exists, loading...");
    let file = match File::open(cache_file_path) {
        Ok(file) => file,
        Err(err) => {
            let cache_file_string = path_to_string(&cache_file_path)?;

            return Err(
                CacheError::FailedToOpenCacheFile(cache_file_string, err.to_string()).into(),
            );
        }
    };

    let cache: Cache = match serde_json::from_reader(file) {
        Ok(cache) => cache,
        Err(err) => {
            error!(sugar_config.logger, "Failed to parse cache file: {}", err);
            return Err(CacheError::CacheFileWrongFormat(err.to_string()).into());
        }
    };

    let candy_machine_pubkey = Pubkey::from_str(&cache.program.candy_machine)?;
    let pid = CANDY_MACHINE_V2.parse().expect("Failed to parse PID");
    let client = setup_client(&sugar_config)?;
    let program = client.program(pid);
    // let payer = program.payer();

    let data = match program.rpc().get_account_data(&candy_machine_pubkey) {
        Ok(account_data) => account_data,
        Err(err) => {
            return Err(VerifyError::FailedToGetAccountData(err.to_string()).into());
        }
    };

    // let candy_machine: CandyMachine = CandyMachine::try_deserialize(&mut data.as_slice())?;
    let num_items = cache.items.0.len();
    let mut on_chain_items: Vec<OnChainItem> = Vec::new();

    for i in 0..num_items {
        let name_start =
            CONFIG_ARRAY_START + STRING_LEN_SIZE + CONFIG_LINE_SIZE * i + CONFIG_NAME_OFFSET;
        let name_end = name_start + MAX_NAME_LENGTH;
        let uri_start =
            CONFIG_ARRAY_START + STRING_LEN_SIZE + CONFIG_LINE_SIZE * i + CONFIG_URI_OFFSET;
        let uri_end = uri_start + MAX_URI_LENGTH;

        let name = String::from_utf8(data[name_start..name_end].to_vec())?
            .trim_matches(char::from(0))
            .to_string();
        let uri = String::from_utf8(data[uri_start..uri_end].to_vec())?
            .trim_matches(char::from(0))
            .to_string();

        on_chain_items.push(OnChainItem { name, uri });
    }

    for item in on_chain_items {
        println!("{item:?}");
    }

    Ok(())
}
