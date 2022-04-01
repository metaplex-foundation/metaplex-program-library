use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey};
use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;
use std::str::FromStr;

use mpl_candy_machine::{
    Creator as CandyCreator, EndSettingType as CandyEndSettingType,
    EndSettings as CandyEndSettings, GatekeeperConfig as CandyGatekeeperConfig,
    HiddenSettings as CandyHiddenSettings, WhitelistMintMode as CandyWhitelistMintMode,
    WhitelistMintSettings as CandyWhitelistMintSettings,
};

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

    #[serde(deserialize_with = "to_pubkey")]
    #[serde(serialize_with = "to_string")]
    pub sol_treasury_account: Pubkey,

    #[serde(deserialize_with = "to_option_pubkey")]
    #[serde(serialize_with = "to_option_string")]
    pub spl_token_account: Option<Pubkey>,

    #[serde(deserialize_with = "to_option_pubkey")]
    #[serde(serialize_with = "to_option_string")]
    pub spl_token: Option<Pubkey>,

    pub go_live_date: String,

    pub end_settings: Option<EndSettings>,

    pub whitelist_mint_settings: Option<WhitelistMintSettings>,

    pub hidden_settings: Option<HiddenSettings>,

    pub upload_method: UploadMethod,

    pub retain_authority: bool,

    pub is_mutable: bool,
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
pub fn go_live_date_as_timestamp(go_live_date: &str) -> Result<i64> {
    let go_live_date = chrono::DateTime::parse_from_rfc3339(go_live_date)?;
    Ok(go_live_date.timestamp())
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatekeeperConfig {
    /// The network for the gateway token required
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

    pub fn into_candy_format(&self) -> CandyGatekeeperConfig {
        CandyGatekeeperConfig {
            gatekeeper_network: self.gatekeeper_network,
            expire_on_use: self.expire_on_use,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EndSettingType {
    Date,
    Amount,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub fn into_candy_format(&self) -> CandyEndSettings {
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
    pub fn into_candy_format(&self) -> CandyWhitelistMintSettings {
        CandyWhitelistMintSettings {
            mode: self.mode.into_candy_format(),
            mint: self.mint,
            presale: self.presale,
            discount_price: discount_price_to_lamports(self.discount_price),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WhitelistMintMode {
    BurnEveryTime,
    NeverBurn,
}

impl WhitelistMintMode {
    pub fn into_candy_format(&self) -> CandyWhitelistMintMode {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HiddenSettings {
    name: String,
    uri: String,
    hash: String,
}

impl HiddenSettings {
    pub fn new(name: String, uri: String, hash: String) -> HiddenSettings {
        HiddenSettings { name, uri, hash }
    }
    pub fn into_candy_format(&self) -> CandyHiddenSettings {
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

#[derive(Debug, Clone, Serialize)]
pub enum UploadMethod {
    Metaplex,
    Bundlr,
    Arloader,
}

impl Default for UploadMethod {
    fn default() -> UploadMethod {
        UploadMethod::Bundlr
    }
}

impl FromStr for UploadMethod {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "metaplex" => Ok(UploadMethod::Metaplex),
            "bundlr" => Ok(UploadMethod::Bundlr),
            "arloader" => Ok(UploadMethod::Arloader),
            _ => Err(ConfigError::InvalidUploadMethod(s.to_string())),
        }
    }
}

impl ToString for UploadMethod {
    fn to_string(&self) -> String {
        match self {
            UploadMethod::Metaplex => "metaplex".to_string(),
            UploadMethod::Bundlr => "bundlr".to_string(),
            UploadMethod::Arloader => "arloader".to_string(),
        }
    }
}

impl<'de> Deserialize<'de> for UploadMethod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct Creator {
    #[serde(deserialize_with = "to_pubkey")]
    pub address: Pubkey,
    pub share: u8,
}

impl Creator {
    pub fn into_candy_format(&self) -> Result<CandyCreator> {
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
