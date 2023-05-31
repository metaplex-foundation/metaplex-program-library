use super::*;

// Large buffer because the older master editions have two pubkeys in them,
// need to keep two versions same size because the conversion process actually
// changes the same account by rewriting it.
pub const MAX_MASTER_EDITION_LEN: usize = 1 + 9 + 8 + 264;

// The last byte of the account containts the token standard value for
// pNFT assets. This is used to restrict legacy operations on the master
// edition account.
pub const TOKEN_STANDARD_INDEX: usize = MAX_MASTER_EDITION_LEN - 1;

// The second to last byte of the account contains the fee flag, indicating
// if the account has fees available for retrieval.
pub const MASTER_EDITION_FEE_FLAG_INDEX: usize = MAX_MASTER_EDITION_LEN - 2;

pub trait MasterEdition {
    fn key(&self) -> Key;
    fn supply(&self) -> u64;
    fn set_supply(&mut self, supply: u64);
    fn max_supply(&self) -> Option<u64>;
    fn save(&self, account: &AccountInfo) -> ProgramResult;
}

pub fn get_master_edition(account: &AccountInfo) -> Result<Box<dyn MasterEdition>, ProgramError> {
    let version = account.data.borrow()[0];

    // For some reason when converting Key to u8 here, it becomes unreachable. Use direct constant instead.
    let master_edition_result: Result<Box<dyn MasterEdition>, ProgramError> = match version {
        2 => {
            let me = MasterEditionV1::from_account_info(account)?;
            Ok(Box::new(me))
        }
        6 => {
            let me = MasterEditionV2::from_account_info(account)?;
            Ok(Box::new(me))
        }
        _ => Err(MetadataError::DataTypeMismatch.into()),
    };

    master_edition_result
}

#[macro_export]
macro_rules! edition_seeds {
    ($mint:expr) => {{
        let path = vec![
            "metadata".as_bytes(),
            $crate::ID.as_ref(),
            $mint.as_ref(),
            "edition".as_bytes(),
        ];
        let (_, bump) = Pubkey::find_program_address(&path, &$crate::ID);
        &[
            "metadata".as_bytes(),
            $crate::ID.as_ref(),
            $mint.as_ref(),
            "edition".as_bytes(),
            &[bump],
        ]
    }};
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize, ShankAccount)]
pub struct MasterEditionV2 {
    pub key: Key,

    pub supply: u64,

    pub max_supply: Option<u64>,
}

impl Default for MasterEditionV2 {
    fn default() -> Self {
        MasterEditionV2 {
            key: Key::MasterEditionV2,
            supply: 0,
            max_supply: Some(0),
        }
    }
}

impl TokenMetadataAccount for MasterEditionV2 {
    fn key() -> Key {
        Key::MasterEditionV2
    }

    fn size() -> usize {
        MAX_MASTER_EDITION_LEN
    }
}

impl MasterEdition for MasterEditionV2 {
    fn key(&self) -> Key {
        self.key
    }

    fn supply(&self) -> u64 {
        self.supply
    }

    fn set_supply(&mut self, supply: u64) {
        self.supply = supply;
    }

    fn max_supply(&self) -> Option<u64> {
        self.max_supply
    }

    fn save(&self, account: &AccountInfo) -> ProgramResult {
        let mut storage = &mut account.data.borrow_mut()[..MASTER_EDITION_FEE_FLAG_INDEX];
        BorshSerialize::serialize(self, &mut storage)?;
        Ok(())
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize, ShankAccount)]
pub struct MasterEditionV1 {
    pub key: Key,

    pub supply: u64,

    pub max_supply: Option<u64>,

    /// Can be used to mint tokens that give one-time permission to mint a single limited edition.
    pub printing_mint: Pubkey,

    /// If you don't know how many printing tokens you are going to need, but you do know
    /// you are going to need some amount in the future, you can use a token from this mint.
    /// Coming back to token metadata with one of these tokens allows you to mint (one time)
    /// any number of printing tokens you want. This is used for instance by Auction Manager
    /// with participation NFTs, where we dont know how many people will bid and need participation
    /// printing tokens to redeem, so we give it ONE of these tokens to use after the auction is over,
    /// because when the auction begins we just dont know how many printing tokens we will need,
    /// but at the end we will. At the end it then burns this token with token-metadata to
    /// get the printing tokens it needs to give to bidders. Each bidder then redeems a printing token
    /// to get their limited editions.
    pub one_time_printing_authorization_mint: Pubkey,
}

impl TokenMetadataAccount for MasterEditionV1 {
    fn key() -> Key {
        Key::MasterEditionV1
    }

    fn size() -> usize {
        MAX_MASTER_EDITION_LEN
    }
}

impl MasterEdition for MasterEditionV1 {
    fn key(&self) -> Key {
        self.key
    }

    fn supply(&self) -> u64 {
        self.supply
    }

    fn max_supply(&self) -> Option<u64> {
        self.max_supply
    }

    fn set_supply(&mut self, supply: u64) {
        self.supply = supply;
    }

    fn save(&self, account: &AccountInfo) -> ProgramResult {
        let mut storage = &mut account.data.borrow_mut()[..MASTER_EDITION_FEE_FLAG_INDEX];
        BorshSerialize::serialize(self, &mut storage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use borsh::BorshSerialize;
    use solana_program::account_info::AccountInfo;
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::{
        error::MetadataError,
        state::{Key, MasterEditionV2, Metadata, TokenMetadataAccount},
        ID,
    };

    #[test]
    fn successfully_deserialize() {
        let expected_data = MasterEditionV2::default();

        let mut buf = Vec::new();
        expected_data.serialize(&mut buf).unwrap();
        MasterEditionV2::pad_length(&mut buf).unwrap();

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

        let data = MasterEditionV2::from_account_info(&account_info).unwrap();
        assert_eq!(data.key, Key::MasterEditionV2);
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

        let error = MasterEditionV2::from_account_info(&account_info).unwrap_err();
        assert_eq!(error, MetadataError::DataTypeMismatch.into());
    }
}
