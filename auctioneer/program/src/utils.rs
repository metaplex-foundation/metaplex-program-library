use anchor_lang::prelude::*;

use crate::{errors::*, sell::config::*};

pub fn assert_auction_active(listing_config: &Account<ListingConfig>) -> Result<()> {
    msg!(
        "Start: {:?}, End: {:?}",
        listing_config.start_time,
        listing_config.end_time
    );

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    msg!("Current: {:?}", current_timestamp);

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

    msg!("Current: {:?}", current_timestamp);

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
        msg!(
            "{:?} is not higher than {:?}",
            new_bid_price,
            listing_config.highest_bid.amount
        );
        return err!(AuctioneerError::BidTooLow);
    }

    msg!(
        "{:?} is higher than {:?}",
        new_bid_price,
        listing_config.highest_bid.amount
    );

    Ok(())
}
