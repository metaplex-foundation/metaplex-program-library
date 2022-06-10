use anchor_lang::prelude::*;

use crate::{errors::*, sell::config::*};

pub fn assert_auction_active(listing_config: &Account<ListingConfig>) -> Result<()> {
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    if current_timestamp < listing_config.start_time {
        return err!(AuctioneerError::AuctionNotStarted);
    } else if current_timestamp > listing_config.end_time {
        return err!(AuctioneerError::AuctionEnded);
    }

    Ok(())
}

pub fn assert_auction_over(listing_config: &Account<ListingConfig>) -> Result<()> {
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    if current_timestamp < listing_config.end_time {
        return err!(AuctioneerError::AuctionActive);
    }

    Ok(())
}

pub fn assert_higher_bid(
    listing_config: &Account<ListingConfig>,
    new_bid_price: u64,
) -> Result<()> {
    if new_bid_price <= listing_config.highest_bid.amount {
        return err!(AuctioneerError::BidTooLow);
    }

    Ok(())
}

pub fn assert_highest_bidder(
    listing_config: &Account<ListingConfig>,
    buyer_trade_state: Pubkey,
) -> Result<()> {
    if buyer_trade_state != listing_config.highest_bid.buyer_trade_state {
        return err!(AuctioneerError::NotHighestBidder);
    }

    Ok(())
}
