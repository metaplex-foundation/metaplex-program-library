use mpl_token_metadata::solana_program::msg;
use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Error, Debug, FromPrimitive, Copy, Clone, Eq, PartialEq)]
pub enum TrifleError {
    /// 0 - Numerical Overflow
    #[error("Numerical Overflow")]
    NumericalOverflow,

    /// Invalid account
    #[error("Invalid account")]
    InvalidAccount,

    /// Invalid Escrow Constraint Model
    #[error("Invalid Escrow Constraint Model")]
    InvalidEscrowConstraintModel,

    /// Invalid Escrow Constraint
    #[error("Invalid Escrow Constraint")]
    InvalidEscrowConstraint,

    /// Escrow Constraint Violation
    #[error("Escrow Constraint Violation")]
    EscrowConstraintViolation,

    /// Invalid Update Authority
    #[error("Invalid Update Authority")]
    InvalidUpdateAuthority,

    /// Failed to create pubkey
    #[error("Failed to create pubkey")]
    FailedToCreatePubkey,

    /// Data type mismatch
    #[error("Data type mismatch")]
    DataTypeMismatch,

    /// Constraint already exists
    #[error("Constraint already exists")]
    ConstraintAlreadyExists,

    /// Token limit exceeded
    #[error("Token Limit Exceeded")]
    TokenLimitExceeded,

    #[error("Failed to find Token Amount")]
    FailedToFindTokenAmount,
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
