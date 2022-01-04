//! Module provide program defined errors

use anchor_lang::prelude::*;

#[error]
pub enum ErrorCode {
    #[msg("No valid signer present")]
    NoValidSignerPresent,
    #[msg("Some string variable is longer than allowed")]
    StringIsTooLong,
    #[msg("Provided supply is gt than available")]
    SupplyIsGtThanAvailable,
    #[msg("Supply is not provided")]
    SupplyIsNotProvided,
    #[msg("Derived key invalid")]
    DerivedKeyInvalid,
}
