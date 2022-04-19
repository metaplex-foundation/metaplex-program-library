use crate::common::*;

use crate::verify::VerifyError;
use indicatif::ProgressIterator;
use std::{thread, time::Duration};

pub struct VerifyArgs {
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
    let sugar_config = match sugar_setup(args.keypair, args.rpc_url) {
        Ok(sugar_config) => sugar_config,
        Err(err) => {
            return Err(SetupError::SugarSetupError(err.to_string()).into());
        }
    };

    let cache_file_path = Path::new(&args.cache);

    if !cache_file_path.exists() {
        return Err(CacheError::CacheFileNotFound(args.cache.clone()).into());
    }

    info!("Cache exists, loading...");
    let file = match File::open(cache_file_path) {
        Ok(file) => file,
        Err(err) => {
            let cache_file_string = path_to_string(cache_file_path)?;

            return Err(
                CacheError::FailedToOpenCacheFile(cache_file_string, err.to_string()).into(),
            );
        }
    };

    let mut cache: Cache = match serde_json::from_reader(file) {
        Ok(cache) => cache,
        Err(err) => {
            error!("Failed to parse cache file: {}", err);
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
    // Should sleep for a total of 1.25 seconds
    let sleep_micros: u64 = 1250000 / num_items as u64;
    let cache_items = &mut cache.items.0;

    let mut invalid_items: Vec<CacheItem> = Vec::new();

    (0..num_items).into_iter().progress().for_each(|i| {
        let name_start =
            CONFIG_ARRAY_START + STRING_LEN_SIZE + CONFIG_LINE_SIZE * i + CONFIG_NAME_OFFSET;
        let name_end = name_start + MAX_NAME_LENGTH;
        let uri_start =
            CONFIG_ARRAY_START + STRING_LEN_SIZE + CONFIG_LINE_SIZE * i + CONFIG_URI_OFFSET;
        let uri_end = uri_start + MAX_URI_LENGTH;
        let name_error = format!("Cache file failed to decode name at line item {}", i);
        let name = String::from_utf8(data[name_start..name_end].to_vec())
            .expect(&name_error)
            .trim_matches(char::from(0))
            .to_string();
        let uri_error = format!("Cache file failed to decode uri at line item {}", i);
        let uri = String::from_utf8(data[uri_start..uri_end].to_vec())
            .expect(&uri_error)
            .trim_matches(char::from(0))
            .to_string();

        let on_chain_item = OnChainItem { name, uri };
        let cache_item = cache_items
            .get_mut(&i.to_string())
            .expect("Failed to get item from config.");

        if !items_match(cache_item, &on_chain_item) {
            cache_item.on_chain = false;
            invalid_items.push(cache_item.clone());
        }

        thread::sleep(Duration::from_micros(sleep_micros));
    });

    cache.write_to_file(cache_file_path)?;

    if !invalid_items.is_empty() {
        println!("Invalid items found: ");
        for item in invalid_items {
            println!("{:?}", item);
        }
        println!("Cache updated. Rerun `deploy`.");
    } else {
        println!("All items checked out. You're good to go!");
    }

    Ok(())
}

fn items_match(cache_item: &CacheItem, on_chain_item: &OnChainItem) -> bool {
    cache_item.name == on_chain_item.name && cache_item.metadata_link == on_chain_item.uri
}
