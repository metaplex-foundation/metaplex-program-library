use anchor_lang::prelude::*;

use crate::{
    errors::ListingRewardsError,
    state::base::{Listing, Offer},
};

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
