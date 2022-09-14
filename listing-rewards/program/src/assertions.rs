use anchor_lang::prelude::*;

use crate::{
    errors::ListingRewardsError,
    state::{Listing, Offer, RewardCenter},
};

pub fn assert_listing_reward_redemption_eligibility(
    listing: &Account<Listing>,
    _: &Account<RewardCenter>,
) -> Result<()> {
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    let eligibility_timestamp = listing.created_at;

    if eligibility_timestamp >= current_timestamp || listing.purchase_ticket.is_some() {
        return Ok(());
    }

    err!(ListingRewardsError::IneligibaleForRewards)
}

pub fn assert_listing_init_eligibility(listing: &Account<Listing>) -> Result<()> {
    if listing.is_initialized
        && (listing.canceled_at.is_none() && listing.purchase_ticket.is_none())
    {
        return err!(ListingRewardsError::ListingAlreadyExists);
    }

    Ok(())
}

pub fn assert_offer_init_eligibility(offer: &Account<Offer>) -> Result<()> {
    if offer.is_initialized && (offer.canceled_at.is_none() && offer.purchase_ticket.is_none()) {
        return err!(ListingRewardsError::OfferAlreadyExists);
    }
    Ok(())
}
