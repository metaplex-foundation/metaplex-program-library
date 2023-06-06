use super::*;
use crate::{
    assertions::{
        collection::assert_collection_update_is_valid, metadata::assert_data_valid,
        uses::assert_valid_use,
    },
    instruction::{CollectionDetailsToggle, CollectionToggle, RuleSetToggle, UpdateArgs},
    utils::{clean_write_metadata, puff_out_data_fields},
};

pub const MAX_NAME_LENGTH: usize = 32;

pub const MAX_SYMBOL_LENGTH: usize = 10;

pub const MAX_URI_LENGTH: usize = 200;

pub const MAX_METADATA_LEN: usize = 1 // key 
+ 32             // update auth pubkey
+ 32             // mint pubkey
+ MAX_DATA_SIZE
+ 1              // primary sale
+ 1              // mutable
+ 9              // nonce (pretty sure this only needs to be 2)
+ 2              // token standard
+ 34             // collection
+ 18             // uses
+ 10             // collection details
+ 33             // programmable config
+ 75; // Padding

pub const MAX_DATA_SIZE: usize = 4
    + MAX_NAME_LENGTH
    + 4
    + MAX_SYMBOL_LENGTH
    + 4
    + MAX_URI_LENGTH
    + 2
    + 1
    + 4
    + MAX_CREATOR_LIMIT * MAX_CREATOR_LEN;

// The last byte of the account contains the fee flag, indicating
// if the account has fees available for retrieval.
pub const METADATA_FEE_FLAG_INDEX: usize = MAX_METADATA_LEN - 1;

#[macro_export]
macro_rules! metadata_seeds {
    ($mint:expr) => {{
        let path = vec!["metadata".as_bytes(), $crate::ID.as_ref(), $mint.as_ref()];
        let (_, bump) = Pubkey::find_program_address(&path, &$crate::ID);
        &[
            "metadata".as_bytes(),
            $crate::ID.as_ref(),
            $mint.as_ref(),
            &[bump],
        ]
    }};
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(Clone, BorshSerialize, Debug, PartialEq, Eq, ShankAccount)]
pub struct Metadata {
    /// Account discriminator.
    pub key: Key,
    /// Address of the update authority.
    #[cfg_attr(feature = "serde-feature", serde(with = "As::<DisplayFromStr>"))]
    pub update_authority: Pubkey,
    /// Address of the mint.
    #[cfg_attr(feature = "serde-feature", serde(with = "As::<DisplayFromStr>"))]
    pub mint: Pubkey,
    /// Asset data.
    pub data: Data,
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
    /// Collection Details
    pub collection_details: Option<CollectionDetails>,
    /// Programmable Config
    pub programmable_config: Option<ProgrammableConfig>,
}

impl Metadata {
    pub fn save(&self, data: &mut [u8]) -> Result<(), BorshError> {
        let mut bytes = Vec::with_capacity(MAX_METADATA_LEN);
        BorshSerialize::serialize(&self, &mut bytes)?;
        data[..bytes.len()].copy_from_slice(&bytes);
        Ok(())
    }

    pub(crate) fn update_v1<'a>(
        &mut self,
        args: UpdateArgs,
        update_authority: &AccountInfo<'a>,
        metadata: &AccountInfo<'a>,
        token: Option<TokenAccount>,
        token_standard: TokenStandard,
    ) -> ProgramResult {
        // Update the token standard if it is changed.
        self.token_standard = Some(token_standard);

        // Only the Update Authority can update this section.
        match &args {
            UpdateArgs::V1 {
                uses,
                collection_details,
                ..
            }
            | UpdateArgs::AsUpdateAuthorityV2 {
                uses,
                collection_details,
                ..
            } => {
                if uses.is_some() {
                    let uses_option = uses.clone().to_option();
                    // If already None leave it as None.
                    assert_valid_use(&uses_option, &self.uses)?;
                    self.uses = uses_option;
                }

                if let CollectionDetailsToggle::Set(collection_details) = collection_details {
                    // only unsized collections can have the size set, and only once.
                    if self.collection_details.is_some() {
                        return Err(MetadataError::SizedCollection.into());
                    }

                    self.collection_details = Some(collection_details.clone());
                }
            }
            _ => (),
        }

        // Update Authority or Data Delegates can update this section.  Note this section is before
        // the section that updates `is_mutable` so that both `data` and `is_mutable` can be updated
        // in the same instruction.
        match &args {
            UpdateArgs::V1 { data, .. }
            | UpdateArgs::AsUpdateAuthorityV2 { data, .. }
            | UpdateArgs::AsDataDelegateV2 { data, .. }
            | UpdateArgs::AsDataItemDelegateV2 { data, .. } => {
                if let Some(data) = data {
                    if !self.is_mutable {
                        return Err(MetadataError::DataIsImmutable.into());
                    }

                    assert_data_valid(
                        data,
                        update_authority.key,
                        self,
                        false,
                        update_authority.is_signer,
                    )?;
                    self.data = data.clone();
                }
            }
            _ => (),
        }

        // Update Authority or Authority Item Delegate can update this section.
        match &args {
            UpdateArgs::V1 {
                new_update_authority,
                primary_sale_happened,
                is_mutable,
                ..
            }
            | UpdateArgs::AsUpdateAuthorityV2 {
                new_update_authority,
                primary_sale_happened,
                is_mutable,
                ..
            }
            | UpdateArgs::AsAuthorityItemDelegateV2 {
                new_update_authority,
                primary_sale_happened,
                is_mutable,
                ..
            } => {
                if let Some(authority) = new_update_authority {
                    self.update_authority = *authority;
                }

                if let Some(primary_sale) = primary_sale_happened {
                    // If received primary_sale is true, flip to true.
                    if *primary_sale || !self.primary_sale_happened {
                        self.primary_sale_happened = *primary_sale
                    } else {
                        return Err(MetadataError::PrimarySaleCanOnlyBeFlippedToTrue.into());
                    }
                }

                if let Some(mutable) = is_mutable {
                    // If received value is false, flip to false.
                    if !mutable || self.is_mutable {
                        self.is_mutable = *mutable
                    } else {
                        return Err(MetadataError::IsMutableCanOnlyBeFlippedToFalse.into());
                    }
                }
            }
            _ => (),
        }

        // Update Authority or Collection Delegates can update this section.
        match &args {
            UpdateArgs::V1 { collection, .. }
            | UpdateArgs::AsUpdateAuthorityV2 { collection, .. }
            | UpdateArgs::AsCollectionDelegateV2 { collection, .. }
            | UpdateArgs::AsCollectionItemDelegateV2 { collection, .. } => match collection {
                // if the Collection data is 'Set', only allow updating if it is unverified
                // or if it exactly matches the existing collection info; if the Collection data
                // is 'Clear', then only set to 'None' if it is unverified.
                CollectionToggle::Set(_) => {
                    let collection_option = collection.clone().to_option();
                    assert_collection_update_is_valid(false, &self.collection, &collection_option)?;
                    self.collection = collection_option;
                }
                CollectionToggle::Clear => {
                    if let Some(current_collection) = self.collection.as_ref() {
                        // Can't change a verified collection in this command.
                        if current_collection.verified {
                            return Err(MetadataError::CannotUpdateVerifiedCollection.into());
                        }
                        // If it's unverified, it's ok to set to None.
                        self.collection = None;
                    }
                }
                CollectionToggle::None => { /* nothing to do */ }
            },
            _ => (),
        };

        // Update Authority or Programmable Config Delegates can update this section.
        match &args {
            UpdateArgs::V1 { rule_set, .. }
            | UpdateArgs::AsUpdateAuthorityV2 { rule_set, .. }
            | UpdateArgs::AsProgrammableConfigDelegateV2 { rule_set, .. }
            | UpdateArgs::AsProgrammableConfigItemDelegateV2 { rule_set, .. } => {
                // if the rule_set data is either 'Set' or 'Clear', only allow updating if the
                // token standard is equal to `ProgrammableNonFungible` and no SPL delegate is set.
                if matches!(rule_set, RuleSetToggle::Clear | RuleSetToggle::Set(_)) {
                    if token_standard != TokenStandard::ProgrammableNonFungible {
                        return Err(MetadataError::InvalidTokenStandard.into());
                    }

                    // Require the token so we can check if it has a token delegate.
                    let token = token.ok_or(MetadataError::MissingTokenAccount)?;

                    // If the token has a delegate, we cannot update the rule set.
                    if token.delegate.is_some() {
                        return Err(MetadataError::CannotUpdateAssetWithDelegate.into());
                    }

                    self.programmable_config =
                        rule_set
                            .clone()
                            .to_option()
                            .map(|rule_set| ProgrammableConfig::V1 {
                                rule_set: Some(rule_set),
                            });
                }
            }
            _ => (),
        };

        // Re-serialize metadata.
        puff_out_data_fields(self);
        clean_write_metadata(self, metadata)
    }

    pub fn into_asset_data(self) -> AssetData {
        let mut asset_data = AssetData::new(
            self.token_standard.unwrap_or(TokenStandard::NonFungible),
            self.data.name,
            self.data.symbol,
            self.data.uri,
        );
        asset_data.seller_fee_basis_points = self.data.seller_fee_basis_points;
        asset_data.creators = self.data.creators;
        asset_data.primary_sale_happened = self.primary_sale_happened;
        asset_data.is_mutable = self.is_mutable;
        asset_data.collection = self.collection;
        asset_data.uses = self.uses;
        asset_data.collection_details = self.collection_details;
        asset_data.rule_set =
            if let Some(ProgrammableConfig::V1 { rule_set }) = self.programmable_config {
                rule_set
            } else {
                None
            };

        asset_data
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            key: Key::MetadataV1,
            update_authority: Pubkey::default(),
            mint: Pubkey::default(),
            data: Data::default(),
            primary_sale_happened: false,
            is_mutable: false,
            edition_nonce: None,
            token_standard: None,
            collection: None,
            uses: None,
            collection_details: None,
            programmable_config: None,
        }
    }
}

impl TokenMetadataAccount for Metadata {
    fn key() -> Key {
        Key::MetadataV1
    }

    fn size() -> usize {
        MAX_METADATA_LEN
    }
}

// We have a custom implementation of BorshDeserialize for Metadata because of corrupted metadata issues
// caused by resizing of the Creators array. We use a custom `meta_deser_unchecked` function
// that has fallback values for corrupted fields.
impl borsh::de::BorshDeserialize for Metadata {
    fn deserialize(buf: &mut &[u8]) -> ::core::result::Result<Self, BorshError> {
        let md = meta_deser_unchecked(buf)?;
        Ok(md)
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
/// Represents the print supply of a non-fungible asset.
pub enum PrintSupply {
    /// The asset does not have any prints.
    Zero,
    /// The asset has a limited amount of prints.
    Limited(u64),
    /// The asset has an unlimited amount of prints.
    Unlimited,
}

impl PrintSupply {
    /// Converts the print supply to an option.
    pub fn to_option(&self) -> Option<u64> {
        match self {
            PrintSupply::Zero => Some(0),
            PrintSupply::Limited(supply) => Some(*supply),
            PrintSupply::Unlimited => None,
        }
    }
}

/// Configuration for programmable assets.
#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum ProgrammableConfig {
    V1 {
        /// Programmable authorization rules.
        #[cfg_attr(
            feature = "serde-feature",
            serde(
                deserialize_with = "deser_option_pubkey",
                serialize_with = "ser_option_pubkey"
            )
        )]
        rule_set: Option<Pubkey>,
    },
}

#[cfg(test)]
mod tests {
    use borsh::{BorshDeserialize, BorshSerialize};
    use solana_program::account_info::AccountInfo;
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::{
        error::MetadataError,
        state::{
            CollectionAuthorityRecord, Edition, EditionMarker, Key, MasterEditionV2, Metadata,
            TokenMetadataAccount, UseAuthorityRecord, MAX_METADATA_LEN,
        },
        utils::metadata::tests::{expected_pesky_metadata, pesky_data},
        ID,
    };

    fn pad_metadata_length(metadata: &mut Vec<u8>) {
        let padding_length = MAX_METADATA_LEN - metadata.len();
        metadata.extend(vec![0; padding_length]);
    }

    #[test]
    fn successfully_deserialize_corrupted_metadata() {
        // This should be able to deserialize the corrupted metadata account successfully due to the custom BorshDeserilization
        // implementation for the Metadata struct.
        let expected_metadata = expected_pesky_metadata();
        let mut corrupted_data = pesky_data();

        let metadata = Metadata::deserialize(&mut corrupted_data).unwrap();

        assert_eq!(metadata, expected_metadata);
    }

    #[test]
    fn successfully_deserialize_metadata() {
        let expected_metadata = expected_pesky_metadata();

        let mut buf = Vec::new();
        expected_metadata.serialize(&mut buf).unwrap();
        pad_metadata_length(&mut buf);

        let pubkey = Keypair::new().pubkey();
        let owner = &ID;
        let mut lamports = 1_000_000_000;
        let mut data = buf.clone();

        let md_account_info = AccountInfo::new(
            &pubkey,
            false,
            true,
            &mut lamports,
            &mut data,
            owner,
            false,
            1_000_000_000,
        );

        let md = Metadata::from_account_info(&md_account_info).unwrap();
        assert_eq!(md.key, Key::MetadataV1);
        assert_eq!(md, expected_metadata);
    }

    #[test]
    fn fail_to_deserialize_metadata_with_wrong_owner() {
        let expected_metadata = expected_pesky_metadata();

        let mut buf = Vec::new();
        expected_metadata.serialize(&mut buf).unwrap();
        pad_metadata_length(&mut buf);

        let pubkey = Keypair::new().pubkey();
        let invalid_owner = Keypair::new().pubkey();
        let mut lamports = 1_000_000_000;
        let mut data = buf.clone();

        let md_account_info = AccountInfo::new(
            &pubkey,
            false,
            true,
            &mut lamports,
            &mut data,
            &invalid_owner,
            false,
            1_000_000_000,
        );

        // `from_account_info` should not succeed because this account is not owned
        // by `token-metadata` program.
        let error = Metadata::from_account_info(&md_account_info).unwrap_err();
        assert_eq!(error, MetadataError::IncorrectOwner.into());
    }

    #[test]
    fn fail_to_deserialize_metadata_with_wrong_size() {
        let expected_metadata = expected_pesky_metadata();

        let mut buf = Vec::new();
        expected_metadata.serialize(&mut buf).unwrap();
        // No padding is added to the metadata so it's too short.

        let pubkey = Keypair::new().pubkey();
        let owner = ID;
        let mut lamports = 1_000_000_000;
        let mut data = buf.clone();

        let account_info = AccountInfo::new(
            &pubkey,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            1_000_000_000,
        );

        // `from_account_info` should not succeed because this account is not owned
        // by `token-metadata` program.
        let error = Metadata::from_account_info(&account_info).unwrap_err();
        assert_eq!(error, MetadataError::DataTypeMismatch.into());
    }

    #[test]
    fn fail_to_deserialize_master_edition_into_metadata() {
        let master_edition = MasterEditionV2 {
            key: Key::MasterEditionV2,
            supply: 0,
            max_supply: Some(0),
        };
        let mut buf = Vec::new();
        master_edition.serialize(&mut buf).unwrap();

        let pubkey = Keypair::new().pubkey();
        let owner = &ID;
        let mut lamports = 1_000_000_000;
        let mut data = buf.clone();

        let account_info = AccountInfo::new(
            &pubkey,
            false,
            true,
            &mut lamports,
            &mut data,
            owner,
            false,
            1_000_000_000,
        );

        let err = Metadata::from_account_info(&account_info).unwrap_err();
        assert_eq!(err, MetadataError::DataTypeMismatch.into());
    }

    #[test]
    fn fail_to_deserialize_edition_into_metadata() {
        let parent = Keypair::new().pubkey();
        let edition = 1;

        let edition = Edition {
            key: Key::EditionV1,
            parent,
            edition,
        };

        let mut buf = Vec::new();
        edition.serialize(&mut buf).unwrap();

        let pubkey = Keypair::new().pubkey();
        let owner = &ID;
        let mut lamports = 1_000_000_000;
        let mut data = buf.clone();

        let account_info = AccountInfo::new(
            &pubkey,
            false,
            true,
            &mut lamports,
            &mut data,
            owner,
            false,
            1_000_000_000,
        );

        let err = Metadata::from_account_info(&account_info).unwrap_err();
        assert_eq!(err, MetadataError::DataTypeMismatch.into());
    }

    #[test]
    fn fail_to_deserialize_use_authority_record_into_metadata() {
        let use_record = UseAuthorityRecord {
            key: Key::UseAuthorityRecord,
            allowed_uses: 14,
            bump: 255,
        };

        let mut buf = Vec::new();
        use_record.serialize(&mut buf).unwrap();

        let pubkey = Keypair::new().pubkey();
        let owner = &ID;
        let mut lamports = 1_000_000_000;
        let mut data = buf.clone();

        let account_info = AccountInfo::new(
            &pubkey,
            false,
            true,
            &mut lamports,
            &mut data,
            owner,
            false,
            1_000_000_000,
        );

        let err = Metadata::from_account_info(&account_info).unwrap_err();
        assert_eq!(err, MetadataError::DataTypeMismatch.into());
    }

    #[test]
    fn fail_to_deserialize_collection_authority_record_into_metadata() {
        let collection_record = CollectionAuthorityRecord {
            key: Key::CollectionAuthorityRecord,
            bump: 255,
            update_authority: None,
        };

        let mut buf = Vec::new();
        collection_record.serialize(&mut buf).unwrap();

        let pubkey = Keypair::new().pubkey();
        let owner = &ID;
        let mut lamports = 1_000_000_000;
        let mut data = buf.clone();

        let account_info = AccountInfo::new(
            &pubkey,
            false,
            true,
            &mut lamports,
            &mut data,
            owner,
            false,
            1_000_000_000,
        );

        let err = Metadata::from_account_info(&account_info).unwrap_err();
        assert_eq!(err, MetadataError::DataTypeMismatch.into());
    }

    #[test]
    fn fail_to_deserialize_edition_marker_into_metadata() {
        let edition_marker = EditionMarker {
            key: Key::EditionMarker,
            ledger: [0; 31],
        };

        let mut buf = Vec::new();
        edition_marker.serialize(&mut buf).unwrap();

        let pubkey = Keypair::new().pubkey();
        let owner = &ID;
        let mut lamports = 1_000_000_000;
        let mut data = buf.clone();

        let account_info = AccountInfo::new(
            &pubkey,
            false,
            true,
            &mut lamports,
            &mut data,
            owner,
            false,
            1_000_000_000,
        );

        let err = Metadata::from_account_info(&account_info).unwrap_err();
        assert_eq!(err, MetadataError::DataTypeMismatch.into());
    }
}
