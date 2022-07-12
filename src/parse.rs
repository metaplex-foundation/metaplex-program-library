use std::{env, fs::File, path::Path};

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;

use crate::{config::data::*, program_errors::*};

pub fn parse_solana_config() -> Option<SolanaConfig> {
    let home = if cfg!(unix) {
        env::var_os("HOME").expect("Couldn't find UNIX home key.")
    } else if cfg!(windows) {
        let drive = env::var_os("HOMEDRIVE").expect("Couldn't find Windows home drive key.");
        let path = env::var_os("HOMEPATH").expect("Couldn't find Windows home path key.");
        Path::new(&drive).join(&path).as_os_str().to_owned()
    } else if cfg!(target_os = "macos") {
        env::var_os("HOME").expect("Couldn't find MacOS home key.")
    } else {
        panic!("Unsupported OS!");
    };

    let config_path = Path::new(&home)
        .join(".config")
        .join("solana")
        .join("cli")
        .join("config.yml");

    let conf_file = match File::open(config_path) {
        Ok(f) => f,
        Err(_) => return None,
    };
    serde_yaml::from_reader(&conf_file).ok()
}

pub fn path_to_string(path: &Path) -> Result<String> {
    match path.to_str() {
        Some(s) => Ok(s.to_string()),
        None => Err(anyhow!("Couldn't convert path to string.")),
    }
}

pub fn parse_sugar_errors(msg: &str) -> String {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"(0x[A-Za-z1-9]+)").expect("Failed to compile parse_client_error regex.");
    }
    let mat = RE.find(msg);

    // If there's an RPC error code match in the message, try to parse it, otherwise return the message back.
    match mat {
        Some(m) => {
            let code = msg[m.start()..m.end()].to_string();
            find_external_program_error(code)
        }
        None => msg.to_owned(),
    }
}

fn find_external_program_error(code: String) -> String {
    let code = code.to_uppercase();

    let parsed_code = if code.contains("0X") {
        code.replace("0X", "")
    } else {
        format!("{:X}", code.parse::<i64>().unwrap())
    };

    if let Some(e) = ANCHOR_ERROR.get(&parsed_code) {
        format!("Anchor Error: {e}")
    } else if let Some(e) = METADATA_ERROR.get(&parsed_code) {
        format!("Token Metadata Error: {e}")
    } else if let Some(e) = CANDY_ERROR.get(&parsed_code) {
        format!("Candy Machine Error: {e}")
    } else {
        format!("Unknown error. Code: {code}")
    }
}
