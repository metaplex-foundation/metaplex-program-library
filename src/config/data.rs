use std::{
    fmt::{self, Display},
    str::FromStr,
};

use anchor_client::solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::Keypair,
};
pub use anyhow::{anyhow, Result};
use chrono::DateTime;
use mpl_candy_machine::{
    Creator as CandyCreator, EndSettingType as CandyEndSettingType,
    EndSettings as CandyEndSettings, GatekeeperConfig as CandyGatekeeperConfig,
    HiddenSettings as CandyHiddenSettings, WhitelistMintMode as CandyWhitelistMintMode,
    WhitelistMintSettings as CandyWhitelistMintSettings,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
    pub price: f64,
    pub number: u64,
    pub gatekeeper: Option<GatekeeperConfig>,
    pub creators: Vec<Creator>,

    #[serde(deserialize_with = "to_option_pubkey")]
    #[serde(serialize_with = "to_option_string")]
    pub sol_treasury_account: Option<Pubkey>,

    #[serde(deserialize_with = "to_option_pubkey")]
    #[serde(serialize_with = "to_option_string")]
    pub spl_token_account: Option<Pubkey>,

    #[serde(deserialize_with = "to_option_pubkey")]
    #[serde(serialize_with = "to_option_string")]
    pub spl_token: Option<Pubkey>,

    pub go_live_date: Option<String>,

    pub end_settings: Option<EndSettings>,

    pub whitelist_mint_settings: Option<WhitelistMintSettings>,

    pub hidden_settings: Option<HiddenSettings>,

    pub upload_method: UploadMethod,

    pub retain_authority: bool,

    pub is_mutable: bool,

    pub symbol: String,

    pub seller_fee_basis_points: u16,

    #[serde(serialize_with = "to_option_string")]
    pub aws_s3_bucket: Option<String>,

    #[serde(serialize_with = "to_option_string")]
    pub nft_storage_auth_token: Option<String>,

    #[serde(serialize_with = "to_option_string")]
    pub shdw_storage_account: Option<String>,
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
    let date = DateTime::parse_from_str(go_live_date, "%Y-%m-%d %H:%M:%S %z")?;
    Ok(date.to_rfc2822())
}

pub fn go_live_date_as_timestamp(go_live_date: &Option<String>) -> Result<Option<i64>> {
    if let Some(go_live_date) = go_live_date {
        let format = if let Ok(date) = chrono::DateTime::parse_from_rfc2822(go_live_date) {
            date.timestamp()
        } else if let Ok(date) = chrono::DateTime::parse_from_rfc3339(go_live_date) {
            date.timestamp()
        } else if let Ok(timestamp) = go_live_date.parse::<i64>() {
            timestamp
        } else {
            return Err(anyhow!("Invalid date format. Format must be: RFC2822(Fri, 14 Jul 2022 02:40:00 -0400), RFC3339(2022-02-25T13:00:00Z), or UNIX timestamp."));
        };
        Ok(Some(format))
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

fn to_option_pubkey<'de, D>(deserializer: D) -> Result<Option<Pubkey>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = match Deserialize::deserialize(deserializer) {
        Ok(s) => s,
        Err(_) => return Ok(None),
    };

    let pubkey = Pubkey::from_str(&s).map_err(serde::de::Error::custom)?;
    Ok(Some(pubkey))
}

fn discount_price_to_lamports(discount_price: Option<f64>) -> Option<u64> {
    discount_price.map(|price| (price * LAMPORTS_PER_SOL as f64) as u64)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatekeeperConfig {
    /// The network for the gateway token required
    #[serde(deserialize_with = "to_pubkey")]
    #[serde(serialize_with = "to_string")]
    gatekeeper_network: Pubkey,
    /// Whether or not the token should expire after minting.
    /// The gatekeeper network must support this if true.
    expire_on_use: bool,
}

impl GatekeeperConfig {
    pub fn new(gatekeeper_network: Pubkey, expire_on_use: bool) -> GatekeeperConfig {
        GatekeeperConfig {
            gatekeeper_network,
            expire_on_use,
        }
    }

    pub fn to_candy_format(&self) -> CandyGatekeeperConfig {
        CandyGatekeeperConfig {
            gatekeeper_network: self.gatekeeper_network,
            expire_on_use: self.expire_on_use,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndSettingType {
    Date,
    Amount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndSettings {
    #[serde(rename = "endSettingType")]
    end_setting_type: EndSettingType,
    number: u64,
}

impl EndSettings {
    pub fn new(end_setting_type: EndSettingType, number: u64) -> EndSettings {
        EndSettings {
            end_setting_type,
            number,
        }
    }
    pub fn to_candy_format(&self) -> CandyEndSettings {
        CandyEndSettings {
            end_setting_type: match self.end_setting_type {
                EndSettingType::Date => CandyEndSettingType::Date,
                EndSettingType::Amount => CandyEndSettingType::Amount,
            },
            number: self.number,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhitelistMintSettings {
    mode: WhitelistMintMode,
    #[serde(deserialize_with = "to_pubkey")]
    #[serde(serialize_with = "to_string")]
    mint: Pubkey,
    presale: bool,
    discount_price: Option<f64>,
}

impl WhitelistMintSettings {
    pub fn new(
        mode: WhitelistMintMode,
        mint: Pubkey,
        presale: bool,
        discount_price: Option<f64>,
    ) -> WhitelistMintSettings {
        WhitelistMintSettings {
            mode,
            mint,
            presale,
            discount_price,
        }
    }
    pub fn to_candy_format(&self) -> CandyWhitelistMintSettings {
        CandyWhitelistMintSettings {
            mode: self.mode.to_candy_format(),
            mint: self.mint,
            presale: self.presale,
            discount_price: discount_price_to_lamports(self.discount_price),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WhitelistMintMode {
    BurnEveryTime,
    NeverBurn,
}

impl WhitelistMintMode {
    pub fn to_candy_format(&self) -> CandyWhitelistMintMode {
        match self {
            WhitelistMintMode::BurnEveryTime => CandyWhitelistMintMode::BurnEveryTime,
            WhitelistMintMode::NeverBurn => CandyWhitelistMintMode::NeverBurn,
        }
    }
}

impl FromStr for WhitelistMintMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "burneverytime" => Ok(WhitelistMintMode::BurnEveryTime),
            "neverburn" => Ok(WhitelistMintMode::NeverBurn),
            _ => Err(anyhow::anyhow!("Invalid whitelist mint mode: {}", s)),
        }
    }
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
    pub fn to_candy_format(&self) -> CandyHiddenSettings {
        CandyHiddenSettings {
            name: self.name.clone(),
            uri: self.uri.clone(),
            hash: self
                .hash
                .as_bytes()
                .try_into()
                .expect("Hidden settings hash has to be 32 characters long!"),
        }
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
    pub fn to_candy_format(&self) -> Result<CandyCreator> {
        let creator = CandyCreator {
            address: self.address,
            share: self.share,
            verified: false,
        };

        Ok(creator)
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum Cluster {
    Devnet,
    Mainnet,
}

impl FromStr for Cluster {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "devnet" => Ok(Cluster::Devnet),
            "mainnet" => Ok(Cluster::Mainnet),
            _ => Err(ConfigError::InvalidCluster(s.to_string()).into()),
        }
    }
}

impl ToString for Cluster {
    fn to_string(&self) -> String {
        match self {
            Cluster::Devnet => "devnet".to_string(),
            Cluster::Mainnet => "mainnet".to_string(),
        }
    }
}
