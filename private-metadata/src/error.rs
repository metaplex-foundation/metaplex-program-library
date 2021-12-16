//! Error types

use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

/// Errors that may be returned by the Private Metadata program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum PrivateMetadataError {
    #[error("Invalid Update Authority")]
    InvalidUpdateAuthority,

    #[error("Metadata is Immutable")]
    MetadataIsImmutable,

    #[error("Invalid Metadata Key")]
    InvalidMetadataKey,

    #[error("Invalid Private Metadata Key")]
    InvalidPrivateMetadataKey,

    #[error("Transfer Buffer Already Initialized")]
    BufferAlreadyInitialized,

    #[error("Arithmetic Overflow")]
    Overflow,
}

impl PrintProgramError for PrivateMetadataError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<PrivateMetadataError> for ProgramError {
    fn from(e: PrivateMetadataError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for PrivateMetadataError {
    fn type_of() -> &'static str {
        "Private Metadata Error"
    }
}
