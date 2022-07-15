use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::ValidateParserError;
use crate::validate::{errors, parser};

#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct Metadata {
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub seller_fee_basis_points: u16,
    pub image: String,
    pub animation_url: Option<String>,
    pub external_url: Option<String>,
    pub attributes: Vec<Attribute>,
    pub properties: Property,
}

impl Metadata {
    pub fn validate(&self) -> Result<(), ValidateParserError> {
        parser::check_name(&self.name)?;
        parser::check_symbol(&self.symbol)?;
        parser::check_url(&self.image)?;
        parser::check_seller_fee_basis_points(self.seller_fee_basis_points)?;
        parser::check_creators_shares(&self.properties.creators)?;
        parser::check_creators_addresses(&self.properties.creators)?;

        Ok(())
    }

    pub fn validate_strict(&self) -> Result<(), ValidateParserError> {
        if let Some(animation_url) = &self.animation_url {
            parser::check_url(animation_url)?;
        } else {
            return Err(errors::ValidateParserError::MissingAnimationUrl);
        }

        if let Some(external_url) = &self.external_url {
            parser::check_url(external_url)?;
        } else {
            return Err(errors::ValidateParserError::MissingExternalUrl);
        }

        Self::validate(self)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct Property {
    pub files: Vec<FileAttr>,
    pub creators: Vec<Creator>,
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
