use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use serde::Deserialize;
use std::str::FromStr;

use crate::validate::{errors, parser};

use mpl_candy_machine::Creator as CandyCreator;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Metadata {
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub seller_fee_basis_points: u32,
    pub image: String,
    pub animation_url: Option<String>,
    pub external_url: String,
    pub attributes: Vec<Attribute>,
    pub collection: Option<Collection>,
    pub properties: Property,
}

impl Metadata {
    pub fn validate(self) -> Result<()> {
        parser::check_name(&self.name)?;
        parser::check_symbol(&self.symbol)?;
        parser::check_url(&self.image)?;
        parser::check_url(&self.external_url)?;
        parser::check_seller_fee_basis_points(self.seller_fee_basis_points)?;
        parser::check_creators(&self.properties.creators)?;

        Ok(())
    }

    pub fn validate_strict(self) -> Result<()> {
        if self.animation_url.is_none() {
            return Err(errors::ValidateError::MissingAnimationUrl.into());
        }

        if self.collection.is_none() {
            return Err(errors::ValidateError::MissingCollection.into());
        }

        parser::check_name(&self.name)?;
        parser::check_symbol(&self.symbol)?;
        parser::check_url(&self.image)?;
        parser::check_url(&self.external_url)?;
        parser::check_seller_fee_basis_points(self.seller_fee_basis_points)?;
        parser::check_creators(&self.properties.creators)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Collection {
    pub name: String,
    pub family: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Property {
    pub files: Vec<FileAttr>,
    pub category: String,
    pub creators: Vec<Creator>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Creator {
    pub address: String,
    pub share: u8,
}

impl Creator {
    pub fn into_candy_format(&self) -> Result<CandyCreator> {
        let creator = CandyCreator {
            address: Pubkey::from_str(&self.address)?,
            share: self.share,
            verified: false,
        };

        Ok(creator)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Attribute {
    pub trait_type: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileAttr {
    pub uri: String,
    #[serde(rename = "type")]
    pub file_type: String,
}
