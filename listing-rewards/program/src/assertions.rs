use anchor_lang::prelude::*;
use mpl_auction_house::state::AuctionHouse;

use crate::{reward_center::RewardCenter, errors::ListingRewardsError};


pub fn assert_rewardable_collection_maintainer(
  wallet: Pubkey,
  auction_house: &Box<Account<AuctionHouse>>,
  reward_center: &Account<RewardCenter>
) -> Result<()> {
  if auction_house.authority == wallet {
    return Ok(());
  }

  let collection_oracle = reward_center.collection_oracle.ok_or(ListingRewardsError::InvalidCollectionMaintainer)?;

  require_eq!(collection_oracle, wallet, ListingRewardsError::InvalidCollectionMaintainer);

  Ok(())
}