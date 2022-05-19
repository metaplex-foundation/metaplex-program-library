use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path};

use mpl_candy_machine::ConfigLine;

use crate::common::*;
use crate::mint::pdas::get_candy_machine_creator_pda;

#[derive(Debug, Deserialize, Serialize)]
pub struct Cache {
    pub program: CacheProgram,
    pub items: CacheItems,
    #[serde(skip_deserializing, skip_serializing)]
    pub file_path: String,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            program: CacheProgram::new(),
            items: CacheItems::new(),
            file_path: String::new(),
        }
    }

    pub fn write_to_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let c = serde_json::to_string(&self)?;
        let mut f = fs::File::create(path)?;
        f.write_all(c.as_bytes())?;

        Ok(())
    }

    pub fn sync_file(&mut self) -> Result<()> {
        let file_path = self.file_path.clone();
        self.write_to_file(Path::new(&file_path))
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CacheProgram {
    #[serde(rename = "candyMachine")]
    pub candy_machine: String,
    #[serde(rename = "candyMachineCreator")]
    pub candy_machine_creator: String,
}

impl CacheProgram {
    pub fn new() -> Self {
        CacheProgram {
            candy_machine: String::new(),
            candy_machine_creator: String::new(),
        }
    }

    pub fn new_from_cm(candy_machine: &Pubkey) -> Self {
        let (candy_machine_creator_pda, _creator_bump) =
            get_candy_machine_creator_pda(candy_machine);
        CacheProgram {
            candy_machine: candy_machine.to_string(),
            candy_machine_creator: candy_machine_creator_pda.to_string(),
        }
    }
}

impl Default for CacheProgram {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CacheItems(pub IndexMap<String, CacheItem>);

impl CacheItems {
    pub fn new() -> Self {
        CacheItems(IndexMap::new())
    }
}
impl Default for CacheItems {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CacheItem {
    pub name: String,
    pub image_hash: String,
    pub image_link: String,
    pub metadata_hash: String,
    pub metadata_link: String,
    #[serde(rename = "onChain")]
    pub on_chain: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation_link: Option<String>,
}

impl CacheItem {
    pub fn into_config_line(&self) -> Option<ConfigLine> {
        if !self.on_chain {
            Some(ConfigLine {
                name: self.name.clone(),
                uri: self.metadata_link.clone(),
            })
        } else {
            None
        }
    }
}

pub fn load_cache(cache_file_path: &str, create: bool) -> Result<Cache> {
    let cache_file_path = Path::new(cache_file_path);
    if !cache_file_path.exists() {
        if create {
            // if the cache file does not exist, creates a new Cache object
            let mut cache = Cache::new();
            cache.file_path = path_to_string(cache_file_path)?;
            Ok(cache)
        } else {
            let cache_file_string = path_to_string(cache_file_path)?;
            let error = CacheError::CacheFileNotFound(cache_file_string).into();
            error!("{:?}", error);
            Err(error)
        }
    } else {
        info!("Cache exists, loading...");
        let file = match File::open(cache_file_path) {
            Ok(file) => file,
            Err(err) => {
                let cache_file_string = path_to_string(cache_file_path)?;
                let error =
                    CacheError::FailedToOpenCacheFile(cache_file_string, err.to_string()).into();
                error!("{:?}", error);
                return Err(error);
            }
        };

        let mut cache: Cache = match serde_json::from_reader(file) {
            Ok(cache) => cache,
            Err(err) => {
                let error = CacheError::CacheFileWrongFormat(err.to_string()).into();
                error!("{:?}", error);
                return Err(error);
            }
        };
        cache.file_path = path_to_string(cache_file_path)?;

        Ok(cache)
    }
}
