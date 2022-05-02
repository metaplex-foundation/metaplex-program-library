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
    #[msg("The bid was lower than the highest bid.")]
    BidTooLow,
}
