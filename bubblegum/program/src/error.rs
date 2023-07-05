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
    #[msg("User-provided creator Vec must result in same user-provided creator hash")]
    CreatorHashMismatch,
    #[msg("User-provided metadata must result in same user-provided data hash")]
    DataHashMismatch,
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
    #[msg("Tree creator or tree delegate must sign.")]
    TreeAuthorityIncorrect,
    #[msg("Not enough unapproved mints left")]
    InsufficientMintCapacity,
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
    #[msg("This transaction must be signed by either the leaf owner or leaf delegate")]
    LeafAuthorityMustSign,
    #[msg("Collection Not Compatable with Compression, Must be Sized")]
    CollectionMustBeSized,
    #[msg("Metadata mint does not match collection mint")]
    MetadataMintMismatch,
    #[msg("Invalid collection authority")]
    InvalidCollectionAuthority,
    #[msg("Invalid delegate record pda derivation")]
    InvalidDelegateRecord,
}
