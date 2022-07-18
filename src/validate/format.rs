use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::ValidateParserError;
use crate::validate::parser;

#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct Metadata {
    pub name: String,
    pub symbol: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_fee_basis_points: Option<u16>,
    pub image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_url: Option<String>,
    pub attributes: Vec<Attribute>,
    pub properties: Property,
}

impl Metadata {
    pub fn validate(&self) -> Result<(), ValidateParserError> {
        parser::check_name(&self.name)?;
        parser::check_symbol(&self.symbol)?;
        parser::check_url(&self.image)?;

        // If users are using the old format, we do validation on those values.
        if let Some(sfbp) = &self.seller_fee_basis_points {
            parser::check_seller_fee_basis_points(*sfbp)?;
        }

        if let Some(creators) = &self.properties.creators {
            parser::check_creators_shares(creators)?;
            parser::check_creators_addresses(creators)?;
        }

        if let Some(animation_url) = &self.animation_url {
            parser::check_url(animation_url)?;
        }

        if let Some(external_url) = &self.external_url {
            parser::check_url(external_url)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct Property {
    pub files: Vec<FileAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creators: Option<Vec<Creator>>,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct Creator {
    pub address: String,
    pub share: u16,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct Attribute {
    pub trait_type: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct FileAttr {
    pub uri: String,
    #[serde(rename = "type")]
    pub file_type: String,
}
