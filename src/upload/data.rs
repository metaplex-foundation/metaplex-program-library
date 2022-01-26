use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use slog::Logger;
use std::{collections::HashMap, fs::File, io::Write, path::Path};

use mpl_candy_machine::ConfigLine;

use crate::candy_machine::uuid_from_pubkey;

pub struct UploadArgs {
    pub logger: Logger,
    pub assets_dir: String,
    pub arloader_manifest: String,
    pub config: String,
    pub cache: String,
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ArloaderManifest(pub HashMap<String, ArloaderItem>);

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ArloaderItem {
    pub id: String,
    pub files: Vec<ArloaderFile>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ArloaderFile {
    pub uri: String,
    #[serde(rename = "type")]
    pub mime_type: String,
}
