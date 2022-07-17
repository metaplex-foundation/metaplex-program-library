use anchor_lang::prelude::*;
use mpl_auction_house::state::AuctionHouse;

use crate::{
    MetadataAccount,
    sell::Listing,
    errors::ListingRewardsError, reward_center::RewardCenter,
    rewardable_collection::RewardableCollection,
};

pub fn assert_listing_reward_redemption_eligibility(listing: &Account<Listing>, reward_center: &Account<RewardCenter>) -> Result<()> {
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    let eligibility_timestamp = listing.created_at + reward_center.listing_reward_rules.warmup_seconds;

    if listing.reward_redeemed_at.is_some() {
        return err!(ListingRewardsError::RewardsAlreadyClaimed);
    }

    if eligibility_timestamp >= current_timestamp || listing.purchased_at.is_some() {
        return Ok(());
    }

    err!(ListingRewardsError::IneligibaleForRewards)
}

pub fn assert_belongs_to_rewardable_collection(
    metadata: &Box<Account<MetadataAccount>>,
    rewardable_collection: &Box<Account<RewardableCollection>>
) -> Result<()> {
    let collection = metadata
        .collection
        .as_ref()
        .ok_or(ListingRewardsError::NFTMissingCollection)?;

    require_eq!(collection.key, rewardable_collection.collection, ListingRewardsError::NFTMismatchRewardableCollection);

    Ok(())
}

pub fn assert_rewardable_collection_maintainer(
    wallet: Pubkey,
    auction_house: &Box<Account<AuctionHouse>>,
    reward_center: &Account<RewardCenter>,
) -> Result<()> {
    if auction_house.authority == wallet {
        return Ok(());
    }

    let collection_oracle = reward_center
        .collection_oracle
        .ok_or(ListingRewardsError::InvalidCollectionMaintainer)?;

    require_eq!(
        collection_oracle,
        wallet,
        ListingRewardsError::InvalidCollectionMaintainer
    );

    Ok(())
}
