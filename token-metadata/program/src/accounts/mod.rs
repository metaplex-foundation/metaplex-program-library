mod create;

pub use create::*;
use solana_program::account_info::AccountInfo;

pub struct Context<'a, T> {
    pub accounts: T,
    pub remaining: Vec<&'a AccountInfo<'a>>,
}
