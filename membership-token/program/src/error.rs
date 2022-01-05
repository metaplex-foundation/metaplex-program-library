//! Module provide program defined errors

use anchor_lang::prelude::*;

#[error]
pub enum ErrorCode {
    #[msg("No valid signer present")]
    NoValidSignerPresent,
    #[msg("Some string variable is longer than allowed")]
    StringIsTooLong,
    #[msg("Name string variable is longer than allowed")]
    NameIsTooLong,
    #[msg("Description string variable is longer than allowed")]
    DescriptionIsTooLong,
    #[msg("Provided supply is gt than available")]
    SupplyIsGtThanAvailable,
    #[msg("Supply is not provided")]
    SupplyIsNotProvided,
    #[msg("Derived key invalid")]
    DerivedKeyInvalid,
    #[msg("Market is not started")]
    MarketIsNotStarted,
    #[msg("User reach buy limit")]
    UserReachBuyLimit,
    #[msg("Math overflow")]
    MathOverflow,
}
