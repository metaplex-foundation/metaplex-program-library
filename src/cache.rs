use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path};

use mpl_candy_machine::ConfigLine;

use crate::candy_machine::uuid_from_pubkey;
use crate::common::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Cache {
    pub program: CacheProgram,
    pub items: CacheItems,
    pub env: String,
    #[serde(rename = "cacheName")]
    pub cache_name: String,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            program: CacheProgram::new(),
            items: CacheItems::new(),
            env: String::new(),
            cache_name: String::new(),
        }
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        // let mut f = OpenOptions::new().write(true).create(true).open(path)?;
        let mut f = fs::File::create(path)?;
        let c = serde_json::to_string(&self)?;
        f.write_all(c.as_bytes())?;

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CacheProgram {
    pub uuid: String,
    #[serde(rename = "candyMachine")]
    pub candy_machine: String,
}

impl CacheProgram {
    pub fn new() -> Self {
        CacheProgram {
            uuid: String::new(),
            candy_machine: String::new(),
        }
    }

    pub fn new_from_cm(candy_machine: &Pubkey) -> Self {
        CacheProgram {
            uuid: uuid_from_pubkey(&candy_machine),
            candy_machine: candy_machine.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CacheItems(pub IndexMap<String, CacheItem>);

impl CacheItems {
    pub fn new() -> Self {
        CacheItems(IndexMap::new())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CacheItem {
    pub name: String,
    pub link: String,
    #[serde(rename = "onChain")]
    pub on_chain: bool,
}

impl CacheItem {
    pub fn into_config_line(&self) -> Option<ConfigLine> {
        if !self.on_chain {
            Some(ConfigLine {
                name: self.name.clone(),
                uri: self.link.clone(),
            })
        } else {
            None
        }
    }
}
