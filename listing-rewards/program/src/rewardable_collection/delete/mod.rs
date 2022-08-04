use anchor_lang::prelude::*;
use mpl_auction_house::{self, constants::PREFIX, AuctionHouse};

use crate::{
    assertions::*,
    constants::{REWARDABLE_COLLECTION, REWARD_CENTER},
    errors::ListingRewardsError,
    state::{RewardCenter, RewardableCollection},
};


#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct DeleteRewardableCollectionParams {
    pub collection: Pubkey,
}

/// Accounts for the [`create_rewardable_collection` handler](listing_rewards/fn.create_rewardable_collection.html).
#[derive(Accounts, Clone)]
#[instruction(rewardable_collection_params: DeleteRewardableCollectionParams)]
pub struct DeleteRewardableCollection<'info> {
    /// The wallet of collection maintainer. Either the auction house authority or collection oracle.
    #[account(mut)]
    pub wallet: Signer<'info>,

    /// The auctioneer program PDA running this auction.
    #[account(
        mut,
        constraint = rewardable_collection.is_initialized && rewardable_collection.deleted_at.is_none() @ ListingRewardsError::RewardableCollectionAlreadyDeleted,
        seeds = [
            REWARDABLE_COLLECTION.as_bytes(), 
            reward_center.key().as_ref(), 
            rewardable_collection_params.collection.as_ref()
        ],
        bump = rewardable_collection.bump,
    )]
    pub rewardable_collection: Account<'info, RewardableCollection>,

    /// Auction House instance PDA account.
    #[account(
        seeds = [
            PREFIX.as_bytes(), 
            auction_house.creator.as_ref(), 
            auction_house.treasury_mint.as_ref()
        ], 
        seeds::program=mpl_auction_house::id(), 
        bump = auction_house.bump
    )]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// The auctioneer program PDA running this auction.
    #[account(
        seeds = [
            REWARD_CENTER.as_bytes(), 
            auction_house.key().as_ref()
        ], 
        bump = reward_center.bump
    )]
    pub reward_center: Account<'info, RewardCenter>,
}

pub fn handler(
    ctx: Context<DeleteRewardableCollection>,
    DeleteRewardableCollectionParams { collection }: DeleteRewardableCollectionParams,
) -> Result<()> {
    let rewardable_collection = &mut ctx.accounts.rewardable_collection;
    let reward_center = &ctx.accounts.reward_center;
    let auction_house = &ctx.accounts.auction_house;
    let wallet = &ctx.accounts.wallet;
    let clock = Clock::get()?;

    require_eq!(
        collection,
        rewardable_collection.collection,
        ListingRewardsError::NFTMismatchRewardableCollection
    );

    assert_rewardable_collection_maintainer(wallet.key(), auction_house, reward_center)?;

    rewardable_collection.deleted_at = Some(clock.unix_timestamp);

    Ok(())
}
