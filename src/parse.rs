use std::{env, fs::File, path::Path};

use anyhow::{anyhow, Result};

use crate::config::data::*;

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
