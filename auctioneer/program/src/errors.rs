use anchor_lang::prelude::*;

#[error_code]
pub enum AuctioneerError {
    // 6000
    #[msg("Bump seed not in hash map")]
    BumpSeedNotInHashMap,

    // 6001
    #[msg("Auction has not started yet")]
    AuctionNotStarted,

    // 6002
    #[msg("Auction has ended")]
    AuctionEnded,

    // 6003
    #[msg("Auction has not ended yet")]
    AuctionActive,

    // 6004
    #[msg("The bid was lower than the highest bid")]
    BidTooLow,

    // 6005
    #[msg("The signer must be the Auction House authority")]
    SignerNotAuth,

    // 6006
    #[msg("Execute Sale must be run on the highest bidder")]
    NotHighestBidder,

    // 6007
    #[msg("The bid price must be greater than the reserve price")]
    BelowReservePrice,

    // 6008
    #[msg("The bid must match the highest bid plus the minimum bid increment")]
    BelowBidIncrement,

    // 6009
    #[msg("The highest bidder is not allowed to cancel")]
    CannotCancelHighestBid,
}
