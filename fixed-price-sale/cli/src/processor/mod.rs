//! Module provide instructions builder for `mpl_fixed_price_sale` program.

mod buy;
mod change_market;
mod close_market;
mod create_market;
mod create_store;
mod get_account_state;
mod init_selling_resource;
mod resume_market;
mod suspend_market;
mod withdraw;
pub use buy::*;
pub use change_market::*;
pub use close_market::*;
pub use create_market::*;
pub use create_store::*;
pub use get_account_state::*;
pub use init_selling_resource::*;
pub use resume_market::*;
pub use suspend_market::*;
pub use withdraw::*;

/// Abstract trait to print additional information in tui.
/// Can be implemented while building instruction.
pub trait UiTransactionInfo {
    fn print(&self);
}
