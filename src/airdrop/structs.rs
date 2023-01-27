use std::hash::{Hash, Hasher};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::common::*;

#[derive(Clone, Debug, Eq)]
pub struct SerdePubkey(pub Pubkey);

impl SerdePubkey {
    pub fn new(pubkey: Pubkey) -> Self {
        SerdePubkey(pubkey)
    }
}

impl PartialEq for SerdePubkey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::fmt::Display for SerdePubkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Copy for SerdePubkey {}

impl Hash for SerdePubkey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl FromStr for SerdePubkey {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pubkey = Pubkey::from_str(s).map_err(|e| format!("Error '{e}' while parsing '{s}'"))?;
        Ok(SerdePubkey(pubkey))
    }
}

impl<'de> Deserialize<'de> for SerdePubkey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl Serialize for SerdePubkey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

pub type AirDropTargets = HashMap<SerdePubkey, u64>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransactionResult {
    pub signature: String,
    pub status: bool,
}

pub type AirDropResults = HashMap<SerdePubkey, Vec<TransactionResult>>;
