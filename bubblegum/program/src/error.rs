use anchor_lang::prelude::*;

#[error_code]
pub enum BubblegumError {
    #[msg("Asset Owner Does not match")]
    AssetOwnerMismatch,
    #[msg("PublicKeyMismatch")]
    PublicKeyMismatch,
    #[msg("Hashing Mismatch Within Leaf Schema")]
    HashingMismatch,
    #[msg("Unsupported Schema Version")]
    UnsupportedSchemaVersion,
    #[msg("Creator shares must sum to 100")]
    CreatorShareTotalMustBe100,
    #[msg("No duplicate creator addresses in metadata")]
    DuplicateCreatorAddress,
    #[msg("Creator did not verify the metadata")]
    CreatorDidNotVerify,
    #[msg("Creator not found in creator Vec")]
    CreatorNotFound,
    #[msg("No creators in creator Vec")]
    NoCreatorsPresent,
    #[msg("Creators list too long")]
    CreatorsTooLong,
    #[msg("Name in metadata is too long")]
    MetadataNameTooLong,
    #[msg("Symbol in metadata is too long")]
    MetadataSymbolTooLong,
    #[msg("Uri in metadata is too long")]
    MetadataUriTooLong,
    #[msg("Basis points in metadata cannot exceed 10000")]
    MetadataBasisPointsTooHigh,
    #[msg("Not enough unapproved mints left")]
    InsufficientMintCapacity,
    #[msg("Mint request not approved")]
    MintRequestNotApproved,
    #[msg("Mint authority key does not match request")]
    MintRequestKeyMismatch,
    #[msg("Mint request data has incorrect disciminator")]
    MintRequestDiscriminatorMismatch,
    #[msg("Something went wrong closing mint request")]
    CloseMintRequestError,
    #[msg("NumericalOverflowError")]
    NumericalOverflowError,
    #[msg("Incorrect account owner")]
    IncorrectOwner,
    #[msg("Cannot Verify Collection in this Instruction")]
    CollectionCannotBeVerifiedInThisInstruction,
    #[msg("Collection Not Found on Metadata")]
    CollectionNotFound,
    #[msg("Collection item is already verified.")]
    AlreadyVerified,
    #[msg("Collection item is already unverified.")]
    AlreadyUnverified,
    #[msg("Incorrect leaf metadata update authority.")]
    UpdateAuthorityIncorrect,
}
