use serde::Deserialize;
use std::collections::HashMap;

pub struct UploadArgs {
    pub assets_dir: String,
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
