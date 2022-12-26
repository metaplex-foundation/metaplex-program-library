use num_derive::ToPrimitive;
use solana_program::{account_info::AccountInfo, instruction::AccountMeta};

#[derive(Debug, Clone, ToPrimitive)]
pub enum Operation {
    Delegate,
    Transfer,
    Sale,
    MigrateClass,
    Update,
}

impl ToString for Operation {
    fn to_string(&self) -> String {
        match self {
            Operation::Delegate => "Delegate",
            Operation::Transfer => "Transfer",
            Operation::Sale => "Sale",
            Operation::MigrateClass => "MigrateClass",
            Operation::Update => "Update",
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
