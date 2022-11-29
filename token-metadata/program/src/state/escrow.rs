use super::*;

pub const ESCROW_POSTFIX: &str = "escrow";

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum EscrowAuthority {
    TokenOwner,
    Creator(Pubkey),
}

impl EscrowAuthority {
    pub fn to_seeds(&self) -> Vec<&[u8]> {
        match self {
            EscrowAuthority::TokenOwner => vec![&[0]],
            EscrowAuthority::Creator(creator) => vec![&[1], creator.as_ref()],
        }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
pub struct TokenOwnedEscrow {
    pub key: Key,
    pub base_token: Pubkey,
    pub authority: EscrowAuthority,
    pub bump: u8,
}

impl TokenMetadataAccount for TokenOwnedEscrow {
    fn key() -> Key {
        Key::TokenOwnedEscrow
    }

    fn size() -> usize {
        0
    }

    fn is_correct_account_type(data: &[u8], data_type: Key, _data_size: usize) -> bool {
        let key: Option<Key> = Key::from_u8(data[0]);
        match key {
            Some(key) => key == data_type || key == Key::Uninitialized,
            None => false,
        }
    }
}
