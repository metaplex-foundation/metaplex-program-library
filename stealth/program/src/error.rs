//! Error types

use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

/// Errors that may be returned by the Stealth program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum StealthError {
    #[error("Invalid Update Authority")]
    InvalidUpdateAuthority,

    #[error("Metadata is Immutable")]
    MetadataIsImmutable,

    #[error("Invalid Metadata Key")]
    InvalidMetadataKey,

    #[error("Invalid Stealth Key")]
    InvalidStealthKey,

    #[error("Transfer Buffer Already Initialized")]
    BufferAlreadyInitialized,

    #[error("Arithmetic Overflow")]
    Overflow,

    #[error("Proof Verification Error")]
    ProofVerificationError,

    #[error("Invalid Elgamal Pubkey PDA")]
    InvalidElgamalPubkeyPDA,

    #[error("Invalid Mint Info")]
    InvalidMintInfo,

    #[error("Invalid Token Account Info")]
    InvalidTokenAccountInfo,
}

impl PrintProgramError for StealthError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<StealthError> for ProgramError {
    fn from(e: StealthError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for StealthError {
    fn type_of() -> &'static str {
        "Stealth Error"
    }
}
