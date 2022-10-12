pub mod data;
pub mod errors;
pub mod guard_data;
pub mod parser;

use std::{fmt::Display, str::FromStr};

use anchor_lang::prelude::Pubkey;
pub use data::*;
pub use errors::*;
pub use guard_data::*;
pub use parser::*;
use serde::{Deserialize, Deserializer, Serializer};
use solana_program::native_token::LAMPORTS_PER_SOL;

pub fn price_as_lamports(price: f64) -> u64 {
    (price * LAMPORTS_PER_SOL as f64) as u64
}

pub fn to_string<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Display,
    S: Serializer,
{
    serializer.collect_str(value)
}

fn to_pubkey<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Pubkey::from_str(&s).map_err(serde::de::Error::custom)
}
