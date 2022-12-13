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

    /// 1 - Invalid account
    #[error("Invalid account")]
    InvalidAccount,

    /// 2 - Invalid Escrow Constraint Model
    #[error("Invalid Escrow Constraint Model")]
    InvalidEscrowConstraintModel,

    /// 3 - Invalid Escrow Constraint
    #[error("Invalid Escrow Constraint")]
    InvalidEscrowConstraint,

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

    /// 8 - Constraint already exists
    #[error("Constraint already exists")]
    ConstraintAlreadyExists,

    /// 9 - Token limit exceeded
    #[error("Token Limit Exceeded")]
    TokenLimitExceeded,

    /// 10 - Failed to find the token amount
    #[error("Failed to find Token Amount")]
    FailedToFindTokenAmount,

    /// 11 - The collection metadata is invalid
    #[error("Invalid Collection Metadata")]
    InvalidCollectionMetadata,

    /// 12 - This set of Transfer Effects can not be used together
    #[error("Provided Transfer Effects are not compatible")]
    TransferEffectConflict,

    /// 13 - The freeze authority is not set
    #[error("Freeze Authority Not Set")]
    FreezeAuthorityNotSet,

    /// 14 - Cannot burn Print Edition
    #[error("Cannot burn Print Edition")]
    CannotBurnPrintEdition,

    /// 15 - The constraint key was not found
    #[error("Constraint Key Not Found")]
    ConstraintKeyNotFound,

    /// 16 - The data failed to serialize
    #[error("Failed to serialize")]
    FailedToSerialize,

    /// 17 - Account data borrow failed
    #[error("Failed to borrow account data")]
    FailedToBorrowAccountData,

    /// 18 - Failed to deserialize the collection
    #[error("Failed to deserialize collection")]
    InvalidCollection,

    /// 19 - Only the holder is allowed to perform this action
    #[error("Only the holder is allowed to perform this action")]
    MustBeHolder,
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
