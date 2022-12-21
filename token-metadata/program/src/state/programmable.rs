use num_derive::ToPrimitive;
use solana_program::{account_info::AccountInfo, instruction::AccountMeta};

#[derive(ToPrimitive)]
pub enum Operation {
    Delegate,
    Transfer,
    DelegatedTransfer,
    MigrateClass,
    Update,
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
