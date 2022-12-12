use std::{
    fmt::{self, Display},
    str::FromStr,
};

use anchor_client::solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::Keypair,
};
pub use anyhow::{anyhow, Result};
use chrono::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::CandyGuardData;
use crate::config::errors::*;

pub struct SugarConfig {
    pub keypair: Keypair,
    pub rpc_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SolanaConfig {
    pub json_rpc_url: String,
    pub keypair_path: String,
    pub commitment: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConfigData {
    /// Number of assets available
    pub number: u64,

    /// Symbol for the asset
    pub symbol: String,

    /// Secondary sales royalty basis points (0-10000)
    pub seller_fee_basis_points: u16,

    /// Indicates if the asset is mutable or not (default yes)
    pub is_mutable: bool,

    /// Indicates whether the index generation is sequential or not
    pub is_sequential: bool,

    /// List of creators
    pub creators: Vec<Creator>,

    /// Upload method to use
    pub upload_method: UploadMethod,

    // AWS specific configuration
    pub aws_config: Option<AwsConfig>,

    // NFT.Storage specific configuration
    #[serde(serialize_with = "to_option_string")]
    pub nft_storage_auth_token: Option<String>,

    // Shadow Drive specific configuration
    #[serde(serialize_with = "to_option_string")]
    pub shdw_storage_account: Option<String>,

    // Pinata specific configuration
    pub pinata_config: Option<PinataConfig>,

    /// Hidden setttings
    pub hidden_settings: Option<HiddenSettings>,

    /// Guards configuration
    pub guards: Option<CandyGuardData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    pub bucket: String,
    pub profile: String,
    pub directory: String,
    pub domain: Option<String>,
}

impl AwsConfig {
    pub fn new(
        bucket: String,
        profile: String,
        directory: String,
        domain: Option<String>,
    ) -> AwsConfig {
        AwsConfig {
            bucket,
            profile,
            directory,
            domain,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PinataConfig {
    pub jwt: String,
    pub api_gateway: String,
    pub content_gateway: String,
    pub parallel_limit: Option<u16>,
}

impl PinataConfig {
    pub fn new(jwt: String, api_gateway: String, content_gateway: String) -> PinataConfig {
        PinataConfig {
            jwt,
            api_gateway,
            content_gateway,
            parallel_limit: None,
        }
    }
}

pub fn to_string<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Display,
    S: Serializer,
{
    serializer.collect_str(value)
}

pub fn to_option_string<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Display,
    S: Serializer,
{
    match value {
        Some(v) => serializer.collect_str(&v),
        None => serializer.serialize_none(),
    }
}

pub fn parse_string_as_date(go_live_date: &str) -> Result<String> {
    let date = dateparser::parse_with(
        go_live_date,
        &Local,
        NaiveTime::from_hms_opt(0, 0, 0).ok_or_else(|| anyhow!("Failed to parse go live date"))?,
    )?;

    Ok(date.to_rfc3339())
}

pub fn go_live_date_as_timestamp(go_live_date: &Option<String>) -> Result<Option<i64>> {
    if let Some(go_live_date) = go_live_date {
        let date = dateparser::parse(go_live_date)?;
        Ok(Some(date.timestamp()))
    } else {
        Ok(None)
    }
}

pub fn price_as_lamports(price: f64) -> u64 {
    (price * LAMPORTS_PER_SOL as f64) as u64
}

fn to_pubkey<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Pubkey::from_str(&s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HiddenSettings {
    name: String,
    uri: String,
    hash: String,
}

impl HiddenSettings {
    pub fn new(name: String, uri: String, hash: String) -> HiddenSettings {
        HiddenSettings { name, uri, hash }
    }
    pub fn to_candy_format(&self) -> mpl_candy_machine_core::HiddenSettings {
        mpl_candy_machine_core::HiddenSettings {
            name: self.name.clone(),
            uri: self.uri.clone(),
            hash: self.hash.as_bytes().try_into().unwrap_or([0; 32]),
        }
    }
    pub fn set_hash(&mut self, hash: String) {
        self.hash = hash;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UploadMethod {
    Bundlr,
    #[serde(rename = "aws")]
    AWS,
    NftStorage,
    #[serde(rename = "shdw")]
    SHDW,
    Pinata,
}

impl Display for UploadMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for UploadMethod {
    fn default() -> UploadMethod {
        UploadMethod::Bundlr
    }
}

#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct Creator {
    #[serde(deserialize_with = "to_pubkey")]
    #[serde(serialize_with = "to_string")]
    pub address: Pubkey,
    pub share: u8,
}

impl Creator {
    pub fn to_candy_format(&self) -> Result<mpl_candy_machine_core::Creator> {
        let creator = mpl_candy_machine_core::Creator {
            address: self.address,
            percentage_share: self.share,
            verified: false,
        };

        Ok(creator)
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum Cluster {
    Devnet,
    Mainnet,
    Localnet,
    Unknown,
}

impl FromStr for Cluster {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "devnet" => Ok(Cluster::Devnet),
            "mainnet" => Ok(Cluster::Mainnet),
            "localnet" => Ok(Cluster::Localnet),
            "unknown" => Ok(Cluster::Unknown),
            _ => Err(ConfigError::InvalidCluster(s.to_string()).into()),
        }
    }
}

impl ToString for Cluster {
    fn to_string(&self) -> String {
        match self {
            Cluster::Devnet => "devnet".to_string(),
            Cluster::Mainnet => "mainnet".to_string(),
            Cluster::Localnet => "localnet".to_string(),
            Cluster::Unknown => "unknown".to_string(),
        }
    }
}
