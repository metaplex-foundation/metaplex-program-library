use anchor_lang::prelude::*;

#[error]
pub enum ErrorCode {
    #[msg("PublicKeyMismatch")]
    PublicKeyMismatch,
    #[msg("InvalidMintAuthority")]
    InvalidMintAuthority,
    #[msg("UninitializedAccount")]
    UninitializedAccount,
    #[msg("IncorrectOwner")]
    IncorrectOwner,
    #[msg("PublicKeysShouldBeUnique")]
    PublicKeysShouldBeUnique,
    #[msg("StatementFalse")]
    StatementFalse,
    #[msg("NotRentExempt")]
    NotRentExempt,
    #[msg("NumericalOverflow")]
    NumericalOverflow,
    #[msg("Expected a sol account but got an spl token account instead")]
    ExpectedSolAccount,
    #[msg("Cannot exchange sol for sol")]
    CannotExchangeSOLForSol,
    #[msg("If paying with sol, sol wallet must be signer")]
    SOLWalletMustSign,
    #[msg("Cannot take this action without auction house signing too")]
    CannotTakeThisActionWithoutAuctionHouseSignOff,
    #[msg("No payer present on this txn")]
    NoPayerPresent,
    #[msg("Derived key invalid")]
    DerivedKeyInvalid,
    #[msg("Metadata doesn't exist")]
    MetadataDoesntExist,
    #[msg("Invalid token amount")]
    InvalidTokenAmount,
    #[msg("Both parties need to agree to this sale")]
    BothPartiesNeedToAgreeToSale,
    #[msg("Cannot match free sales unless the auction house or seller signs off")]
    CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoff,
    #[msg("This sale requires a signer")]
    SaleRequiresSigner,
    #[msg("Old seller not initialized")]
    OldSellerNotInitialized,
    #[msg("Seller ata cannot have a delegate set")]
    SellerATACannotHaveDelegate,
    #[msg("Buyer ata cannot have a delegate set")]
    BuyerATACannotHaveDelegate,
    #[msg("No valid signer present")]
    NoValidSignerPresent,
    #[msg("BP must be less than or equal to 10000")]
    InvalidBasisPoints,
    #[msg("The trade state account does not exist")]
    TradeStateDoesntExist,
    #[msg("The trade state is not empty")]
    TradeStateIsNotEmpty,
    #[msg("The receipt is empty")]
    ReceiptIsEmpty,
    #[msg("The instruction does not match")]
    InstructionMismatch,

    #[msg("Invalid Auctioneer for this Auction House instance.")]
    InvalidAuctioneer,

    #[msg("The Auctioneer does not have the correct scope for this action.")]
    InvalidAuctioneerScope,

    #[msg("Must use auctioneer handler.")]
    MustUseAuctioneerHandler,

    #[msg("No Auctioneer program set.")]
    NoAuctioneerProgramSet,
}
