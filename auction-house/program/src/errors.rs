use anchor_lang::prelude::*;

#[error_code]
pub enum AuctionHouseError {
    // 6000
    #[msg("PublicKeyMismatch")]
    PublicKeyMismatch,

    // 6001
    #[msg("InvalidMintAuthority")]
    InvalidMintAuthority,

    // 6002
    #[msg("UninitializedAccount")]
    UninitializedAccount,

    // 6003
    #[msg("IncorrectOwner")]
    IncorrectOwner,

    // 6004
    #[msg("PublicKeysShouldBeUnique")]
    PublicKeysShouldBeUnique,

    // 6005
    #[msg("StatementFalse")]
    StatementFalse,

    // 6006
    #[msg("NotRentExempt")]
    NotRentExempt,

    // 6007
    #[msg("NumericalOverflow")]
    NumericalOverflow,

    // 6008
    #[msg("Expected a sol account but got an spl token account instead")]
    ExpectedSolAccount,

    // 6009
    #[msg("Cannot exchange sol for sol")]
    CannotExchangeSOLForSol,

    // 6010
    #[msg("If paying with sol, sol wallet must be signer")]
    SOLWalletMustSign,

    // 6011
    #[msg("Cannot take this action without auction house signing too")]
    CannotTakeThisActionWithoutAuctionHouseSignOff,

    // 6012
    #[msg("No payer present on this txn")]
    NoPayerPresent,

    // 6013
    #[msg("Derived key invalid")]
    DerivedKeyInvalid,

    // 6014
    #[msg("Metadata doesn't exist")]
    MetadataDoesntExist,

    // 6015
    #[msg("Invalid token amount")]
    InvalidTokenAmount,

    // 6016
    #[msg("Both parties need to agree to this sale")]
    BothPartiesNeedToAgreeToSale,

    // 6017
    #[msg("Cannot match free sales unless the auction house or seller signs off")]
    CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoff,

    // 6018
    #[msg("This sale requires a signer")]
    SaleRequiresSigner,

    // 6019
    #[msg("Old seller not initialized")]
    OldSellerNotInitialized,

    // 6020
    #[msg("Seller ata cannot have a delegate set")]
    SellerATACannotHaveDelegate,

    // 6021
    #[msg("Buyer ata cannot have a delegate set")]
    BuyerATACannotHaveDelegate,

    // 6022
    #[msg("No valid signer present")]
    NoValidSignerPresent,

    // 6023
    #[msg("BP must be less than or equal to 10000")]
    InvalidBasisPoints,

    // 6024
    #[msg("The trade state account does not exist")]
    TradeStateDoesntExist,

    // 6025
    #[msg("The trade state is not empty")]
    TradeStateIsNotEmpty,

    // 6026
    #[msg("The receipt is empty")]
    ReceiptIsEmpty,

    // 6027
    #[msg("The instruction does not match")]
    InstructionMismatch,

    // 6028
    #[msg("Invalid Auctioneer for this Auction House instance.")]
    InvalidAuctioneer,

    // 6029
    #[msg("The Auctioneer does not have the correct scope for this action.")]
    MissingAuctioneerScope,

    // 6030
    #[msg("Must use auctioneer handler.")]
    MustUseAuctioneerHandler,

    // 6031
    #[msg("No Auctioneer program set.")]
    NoAuctioneerProgramSet,

    // 6032
    #[msg("Too many scopes.")]
    TooManyScopes,

    // 6033
    #[msg("Auction House not delegated.")]
    AuctionHouseNotDelegated,

    // 6034
    #[msg("Bump seed not in hash map.")]
    BumpSeedNotInHashMap,

    // 6035
    #[msg("The instruction would drain the escrow below rent exemption threshold")]
    EscrowUnderRentExemption,

    // 6036
    #[msg("Invalid seeds or Auction House not delegated")]
    InvalidSeedsOrAuctionHouseNotDelegated,

    // 6037
    #[msg("The buyer trade state was unable to be initialized.")]
    BuyerTradeStateNotValid,

    // 6038
    #[msg("Partial order size and price must both be provided in a partial buy.")]
    MissingElementForPartialOrder,

    // 6039
    #[msg("Amount of tokens available for purchase is less than the partial order amount.")]
    NotEnoughTokensAvailableForPurchase,

    // 6040
    #[msg("Calculated partial price does not not partial price that was provided.")]
    PartialPriceMismatch,

    // 6041
    #[msg("Insufficient funds in escrow account to purchase.")]
    InsufficientFunds,
}
