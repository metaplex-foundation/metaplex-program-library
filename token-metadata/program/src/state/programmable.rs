use num_derive::ToPrimitive;
use solana_program::{account_info::AccountInfo, instruction::AccountMeta};

use crate::processor::{TransferAuthority, UpdateAuthority};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    Transfer { scenario: TransferAuthority },
    Update { scenario: UpdateAuthority },
}

impl ToString for Operation {
    fn to_string(&self) -> String {
        match self {
            Self::Transfer { scenario } => format!("TRA:{}", scenario),
            Self::Update { scenario } => format!("UPD:{}", scenario),
        }
    }
}

#[derive(Debug, Clone, ToPrimitive)]
pub enum PayloadKey {
    Amount,
    Authority,
    Delegate,
    Destination,
    DestinationSeeds,
    Holder,
    Source,
    SourceSeeds,
    Target,
}

impl ToString for PayloadKey {
    fn to_string(&self) -> String {
        match self {
            PayloadKey::Amount => "Amount",
            PayloadKey::Authority => "Authority",
            PayloadKey::Delegate => "Delegate",
            PayloadKey::Destination => "Destination",
            PayloadKey::DestinationSeeds => "DestinationSeeds",
            PayloadKey::Holder => "Holder",
            PayloadKey::Source => "Source",
            PayloadKey::SourceSeeds => "SourceSeeds",
            PayloadKey::Target => "Target",
        }
        .to_string()
    }
}

pub trait ToAccountMeta {
    fn to_account_meta(&self) -> AccountMeta;
}

impl<'info> ToAccountMeta for AccountInfo<'info> {
    fn to_account_meta(&self) -> AccountMeta {
        AccountMeta {
            pubkey: *self.key,
            is_signer: self.is_signer,
            is_writable: self.is_writable,
        }
    }
}
