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

    #[error("Proof Verification Error")]
    ProofVerificationError,
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


// TODO: use spl-zk-token-sdk
#[derive(Error, Clone, Debug, Eq, PartialEq)]
pub enum ProofError {
    #[error("proof failed to verify")]
    VerificationError,
    #[error("malformed proof")]
    FormatError,
    #[error("number of blinding factors do not match the number of values")]
    WrongNumBlindingFactors,
    #[error("attempted to create a proof with bitsize other than \\(8\\), \\(16\\), \\(32\\), or \\(64\\)")]
    InvalidBitsize,
    #[error("insufficient generators for the proof")]
    InvalidGeneratorsLength,
    #[error(
        "`zk_token_elgamal::pod::ElGamalCiphertext` contains invalid ElGamalCiphertext ciphertext"
    )]
    InconsistentCTData,
}
