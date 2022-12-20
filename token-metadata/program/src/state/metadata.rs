use crate::{
    assertions::{
        collection::assert_collection_update_is_valid, metadata::assert_data_valid,
        uses::assert_valid_use,
    },
    instruction::UpdateArgs,
    utils::{clean_write_metadata, puff_out_data_fields},
};

use super::*;

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
+ 118; // Padding

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

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(Clone, BorshSerialize, Debug, PartialEq, Eq, ShankAccount)]
pub struct Metadata {
    pub key: Key,
    pub update_authority: Pubkey,
    pub mint: Pubkey,
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
    /// Active delegate (for now, only the sale delegate is persisted)
    pub delegate: Option<Pubkey>,
}

impl Metadata {
    pub fn save(&self, data: &mut [u8]) -> Result<(), BorshError> {
        let mut bytes = Vec::with_capacity(MAX_METADATA_LEN);
        BorshSerialize::serialize(&self, &mut bytes)?;
        data[..bytes.len()].copy_from_slice(&bytes);
        Ok(())
    }

    pub fn update_data<'a>(
        &mut self,
        args: UpdateArgs,
        update_authority: &AccountInfo<'a>,
        metadata: &AccountInfo<'a>,
    ) -> ProgramResult {
        let (
            data,
            primary_sale_happened,
            is_mutable,
            _token_standard,
            collection,
            uses,
            _collection_details,
            _programmable_config,
            _delegate_state,
            _authorization_data,
            new_update_authority,
        ) = match args {
            UpdateArgs::V1 {
                data,
                primary_sale_happened,
                is_mutable,
                token_standard,
                collection,
                uses,
                collection_details,
                programmable_config,
                delegate_state,
                authorization_data,
                new_update_authority,
            } => (
                data,
                primary_sale_happened,
                is_mutable,
                token_standard,
                collection,
                uses,
                collection_details,
                programmable_config,
                delegate_state,
                authorization_data,
                new_update_authority,
            ),
        };

        if let Some(data) = data {
            if self.is_mutable {
                assert_data_valid(
                    &data,
                    update_authority.key,
                    self,
                    false,
                    update_authority.is_signer,
                )?;
                self.data = data;

                // If the user passes in Collection data, only allow updating if it's unverified
                // or if it exactly matches the existing collection info.
                // If the user passes in None for the Collection data then only set it if it's unverified.
                if collection.is_some() {
                    assert_collection_update_is_valid(false, &self.collection, &collection)?;
                    self.collection = collection;
                } else if let Some(current_collection) = self.collection.as_ref() {
                    // Can't change a verified collection in this command.
                    if current_collection.verified {
                        return Err(MetadataError::CannotUpdateVerifiedCollection.into());
                    }
                    // If it's unverified, it's ok to set to None.
                    self.collection = collection;
                }

                // If already None leave it as None.
                assert_valid_use(&uses, &self.uses)?;
                self.uses = uses;
            } else {
                return Err(MetadataError::DataIsImmutable.into());
            }
        }

        if let Some(val) = new_update_authority {
            self.update_authority = val;
        }

        if let Some(val) = primary_sale_happened {
            // If received val is true, flip to true.
            if val || !self.primary_sale_happened {
                self.primary_sale_happened = val
            } else {
                return Err(MetadataError::PrimarySaleCanOnlyBeFlippedToTrue.into());
            }
        }

        if let Some(val) = is_mutable {
            // If received value is false, flip to false.
            if !val || self.is_mutable {
                self.is_mutable = val
            } else {
                return Err(MetadataError::IsMutableCanOnlyBeFlippedToFalse.into());
            }
        }

        puff_out_data_fields(self);
        clean_write_metadata(self, metadata)?;

        Ok(())
    }

    pub fn into_asset_data(self) -> AssetData {
        let mut asset_data = AssetData::new(
            self.token_standard.unwrap_or(TokenStandard::NonFungible),
            self.data.name,
            self.data.symbol,
            self.data.uri,
            self.update_authority,
        );
        asset_data.seller_fee_basis_points = self.data.seller_fee_basis_points;
        asset_data.creators = self.data.creators;
        asset_data.primary_sale_happened = self.primary_sale_happened;
        asset_data.is_mutable = self.is_mutable;
        asset_data.edition_nonce = self.edition_nonce;
        asset_data.collection = self.collection;
        asset_data.uses = self.uses;
        asset_data.collection_details = self.collection_details;
        asset_data.programmable_config = self.programmable_config;
        let delegate_state = self.delegate.map(DelegateState::Transfer);
        asset_data.delegate_state = delegate_state;

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
            delegate: None,
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
