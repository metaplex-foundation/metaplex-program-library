use mpl_token_metadata::solana_program::msg;
use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Error, Debug, FromPrimitive)]
pub enum TrifleError {
    /// 0 - Numerical Overflow
    #[error("Numerical Overflow")]
    NumericalOverflow,

    /// 1 - Invalid account
    #[error("Invalid account")]
    InvalidAccount,

    /// 2 - Invalid Escrow Constraint Model
    #[error("Invalid Escrow Constraint Model")]
    InvalidEscrowConstraintModel,

    /// 3 - Invalid Escrow Constraint Index
    #[error("Invalid Escrow Constraint Index")]
    InvalidEscrowConstraintIndex,

    /// 4 - Escrow Constraint Violation
    #[error("Escrow Constraint Violation")]
    EscrowConstraintViolation,

    /// 5 - Invalid Update Authority
    #[error("Invalid Update Authority")]
    InvalidUpdateAuthority,

    /// 6 - Failed to create pubkey
    #[error("Failed to create pubkey")]
    FailedToCreatePubkey,

    /// 7 - Data type mismatch
    #[error("Data type mismatch")]
    DataTypeMismatch,
}

impl From<TrifleError> for ProgramError {
    fn from(e: TrifleError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl PrintProgramError for TrifleError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl<T> DecodeError<T> for TrifleError {
    fn type_of() -> &'static str {
        "Metadata Error"
    }
}
