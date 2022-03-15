//! Module provide instructions builder for `mpl_fixed_price_sale` program.

mod buy;
mod create_market;
mod create_store;
mod get_account_state;
mod init_selling_resource;
pub use buy::*;
pub use create_market::*;
pub use create_store::*;
pub use get_account_state::*;
pub use init_selling_resource::*;

/// Abstract trait to print additional information in tui.
/// Can be implemented while building instruction.
pub trait UiTransactionInfo {
    fn print(&self);
}
