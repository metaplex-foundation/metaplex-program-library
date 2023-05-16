use anchor_lang::prelude::*;
use mpl_token_metadata::state::{Data, Metadata};

use crate::error::BubblegumError;

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum TokenProgramVersion {
    Original,
    Token2022,
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    // In percentages, NOT basis points ;) Watch out!
    pub share: u8,
}

impl Creator {
    pub fn adapt(&self) -> mpl_token_metadata::state::Creator {
        mpl_token_metadata::state::Creator {
            address: self.address,
            verified: self.verified,
            share: self.share,
        }
    }

    pub fn from(args: mpl_token_metadata::state::Creator) -> Self {
        Creator {
            address: args.address,
            verified: args.verified,
            share: args.share,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum TokenStandard {
    NonFungible,        // This is a master edition
    FungibleAsset,      // A token with metadata that can also have attributes
    Fungible,           // A token with simple metadata
    NonFungibleEdition, // This is a limited edition
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum UseMethod {
    Burn,
    Multiple,
    Single,
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct Uses {
    // 17 bytes + Option byte
    pub use_method: UseMethod, //1
    pub remaining: u64,        //8
    pub total: u64,            //8
}

impl Uses {
    pub fn adapt(&self) -> mpl_token_metadata::state::Uses {
        mpl_token_metadata::state::Uses {
            use_method: match self.use_method {
                UseMethod::Burn => mpl_token_metadata::state::UseMethod::Burn,
                UseMethod::Multiple => mpl_token_metadata::state::UseMethod::Multiple,
                UseMethod::Single => mpl_token_metadata::state::UseMethod::Single,
            },
            remaining: self.remaining,
            total: self.total,
        }
    }

    pub fn from(args: &mpl_token_metadata::state::Uses) -> Self {
        Uses {
            use_method: match args.use_method {
                mpl_token_metadata::state::UseMethod::Burn => UseMethod::Burn,
                mpl_token_metadata::state::UseMethod::Multiple => UseMethod::Multiple,
                mpl_token_metadata::state::UseMethod::Single => UseMethod::Single,
            },
            remaining: args.remaining,
            total: args.total,
        }
    }
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct Collection {
    pub verified: bool,
    pub key: Pubkey,
}

impl Collection {
    pub fn adapt(&self) -> mpl_token_metadata::state::Collection {
        mpl_token_metadata::state::Collection {
            verified: self.verified,
            key: self.key,
        }
    }

    pub fn from(args: &mpl_token_metadata::state::Collection) -> Self {
        Collection {
            verified: args.verified,
            key: args.key,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct MetadataArgs {
    /// The name of the asset
    pub name: String,
    /// The symbol for the asset
    pub symbol: String,
    /// URI pointing to JSON representing the asset
    pub uri: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000)
    pub seller_fee_basis_points: u16,
    // Immutable, once flipped, all sales of this metadata are considered secondary.
    pub primary_sale_happened: bool,
    // Whether or not the data struct is mutable, default is not
    pub is_mutable: bool,
    /// nonce for easy calculation of editions, if present
    pub edition_nonce: Option<u8>,
    /// Since we cannot easily change Metadata, we add the new DataV2 fields here at the end.
    pub token_standard: Option<TokenStandard>,
    /// Collection
    pub collection: Option<Collection>,
    /// Uses
    pub uses: Option<Uses>,
    pub token_program_version: TokenProgramVersion,
    pub creators: Vec<Creator>,
}

impl MetadataArgs {
    /// Also performs validation
    pub fn to_metadata(
        self,
        metadata_auth: &Pubkey,
    ) -> std::result::Result<Metadata, BubblegumError> {
        let creators = match self.creators {
            creators if creators.is_empty() => None,
            creators => Some(
                creators
                    .iter()
                    .map(|c| c.adapt())
                    .collect::<Vec<mpl_token_metadata::state::Creator>>(),
            ),
        };
        let data = Data {
            name: self.name,
            symbol: self.symbol,
            uri: self.uri,
            seller_fee_basis_points: self.seller_fee_basis_points,
            creators,
        };
        let token_standard = match self.token_standard {
            Some(TokenStandard::NonFungible) => {
                Ok(Some(mpl_token_metadata::state::TokenStandard::NonFungible))
            }
            Some(TokenStandard::FungibleAsset) => Err(BubblegumError::TokenStandardNotSupported),
            Some(TokenStandard::Fungible) => Err(BubblegumError::TokenStandardNotSupported),
            Some(TokenStandard::NonFungibleEdition) => {
                Err(BubblegumError::TokenStandardNotSupported)
            }
            None => Err(BubblegumError::TokenStandardNotSupported),
        }?;
        Ok(Metadata {
            key: mpl_token_metadata::state::Key::MetadataV1,
            update_authority: metadata_auth.clone(),
            mint: Pubkey::default(),
            data,
            primary_sale_happened: self.primary_sale_happened,
            is_mutable: self.is_mutable,
            edition_nonce: self.edition_nonce,
            token_standard,
            collection: match self.collection {
                Some(c) => Some(c.adapt()),
                None => None,
            },
            uses: match self.uses {
                Some(u) => Some(u.adapt()),
                None => None,
            },
            collection_details: None,
            programmable_config: None,
        })
    }
}
