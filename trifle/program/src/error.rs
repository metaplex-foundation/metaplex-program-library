use mpl_token_metadata::solana_program::msg;
use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Error, Debug, FromPrimitive)]
pub enum TrifleError {
    #[error("Numerical Overflow")]
    NumericalOverflow,
    #[error("Invalid account")]
    InvalidAccount,
    #[error("Invalid Escrow Constraint Model")]
    InvalidEscrowConstraintModel,
    #[error("Invalid Escrow Constraint Index")]
    InvalidEscrowConstraintIndex,
    #[error("Escrow Constraint Violation")]
    EscrowConstraintViolation,
    #[error("Invalid Update Authority")]
    InvalidUpdateAuthority,
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
