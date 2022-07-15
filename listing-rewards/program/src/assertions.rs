use anchor_lang::prelude::*;
use mpl_auction_house::state::AuctionHouse;

use crate::{
    errors::ListingRewardsError, reward_center::RewardCenter,
    rewardable_collection::RewardableCollection,
};

use mpl_token_metadata::state::Metadata;

pub fn assert_belongs_to_rewardable_collection(
    metadata: Metadata,
    rewardable_collection: &Box<Account<RewardableCollection>>
) -> Result<()> {
    let collection = metadata
        .collection
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
