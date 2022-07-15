pub mod assertions;
pub mod constants;
pub mod errors;
pub mod offers;
pub mod pda;
pub mod reward_center;
pub mod rewardable_collection;
pub mod sell;

use anchor_lang::prelude::*;

use crate::{
    offers::create_offer::{create_offer as c_offer, *},
    reward_center::*,
    rewardable_collection::*,
    sell::*,
};

declare_id!("rwdLstiU8aJU1DPdoPtocaNKApMhCFdCg283hz8dd3u");

#[program]
pub mod listing_rewards {
    use super::*;

    pub fn create_reward_center(
        ctx: Context<CreateRewardCenter>,
        reward_center_params: CreateRewardCenterParams,
    ) -> Result<()> {
        reward_center::create_reward_center(ctx, reward_center_params)
    }

    pub fn create_rewardable_collection(
        ctx: Context<CreateRewardableCollection>,
        rewardable_collection_params: CreateRewardableCollectionParams,
    ) -> Result<()> {
        rewardable_collection::create_rewardable_collection(ctx, rewardable_collection_params)
    }

    pub fn sell(ctx: Context<Sell>, sell_params: SellParams) -> Result<()> {
        sell::sell(ctx, sell_params)
    }

    pub fn create_offer(
        ctx: Context<CreateOffer>,
        create_offer_params: CreateOfferParams,
    ) -> Result<()> {
        c_offer(ctx, create_offer_params)
    }
}
