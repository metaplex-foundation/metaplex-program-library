use anchor_lang::prelude::*;

use crate::{
    errors::ListingRewardsError,
    state::{Listing, RewardCenter},
};

pub fn assert_listing_reward_redemption_eligibility(
    listing: &Account<Listing>,
    _: &Account<RewardCenter>,
) -> Result<()> {
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    let eligibility_timestamp = listing.created_at;

    if listing.reward_redeemed_at.is_some() {
        return err!(ListingRewardsError::RewardsAlreadyClaimed);
    }

    if eligibility_timestamp >= current_timestamp || listing.purchased_at.is_some() {
        return Ok(());
    }

    err!(ListingRewardsError::IneligibaleForRewards)
}
