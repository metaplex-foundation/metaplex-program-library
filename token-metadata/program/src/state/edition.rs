use super::*;

pub const MAX_EDITION_LEN: usize = 1 + 32 + 8 + 200;

// The last byte of the account contains the token standard value for
// pNFT assets. This is used to restrict legacy operations on the master
// edition account.
pub const TOKEN_STANDARD_INDEX_EDITION: usize = MAX_EDITION_LEN - 1;

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize, ShankAccount)]
/// All Editions should never have a supply greater than 1.
/// To enforce this, a transfer mint authority instruction will happen when
/// a normal token is turned into an Edition, and in order for a Metadata update authority
/// to do this transaction they will also need to sign the transaction as the Mint authority.
pub struct Edition {
    pub key: Key,

    /// Points at MasterEdition struct
    #[cfg_attr(feature = "serde-feature", serde(with = "As::<DisplayFromStr>"))]
    pub parent: Pubkey,

    /// Starting at 0 for master record, this is incremented for each edition minted.
    pub edition: u64,
}

impl Default for Edition {
    fn default() -> Self {
        Edition {
            key: Key::EditionV1,
            parent: Pubkey::default(),
            edition: 0,
        }
    }
}

impl TokenMetadataAccount for Edition {
    fn key() -> Key {
        Key::EditionV1
    }

    fn size() -> usize {
        MAX_EDITION_LEN
    }
}

#[cfg(test)]
mod tests {
    use borsh::BorshSerialize;
    use solana_program::account_info::AccountInfo;
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::{
        error::MetadataError,
        state::{Edition, Key, Metadata, TokenMetadataAccount},
        ID,
    };

    #[test]
    fn successfully_deserialize_edition() {
        let expected_data = Edition::default();

        let mut buf = Vec::new();
        expected_data.serialize(&mut buf).unwrap();
        Edition::pad_length(&mut buf).unwrap();

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

        let data = Edition::from_account_info(&account_info).unwrap();
        assert_eq!(data.key, Key::EditionV1);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn deserializing_wrong_account_type_fails() {
        let wrong_type = Metadata::default();

        let mut buf = Vec::new();
        wrong_type.serialize(&mut buf).unwrap();
        Metadata::pad_length(&mut buf).unwrap();

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

        let error = Edition::from_account_info(&account_info).unwrap_err();
        assert_eq!(error, MetadataError::DataTypeMismatch.into());
    }
}
