pub mod assertions;
pub mod constants;
pub mod cpi;
pub mod errors;
pub mod execute_sale;
pub mod listings;
pub mod offers;
pub mod pda;
pub mod redeem_rewards;
pub mod reward_center;
pub mod state;

use anchor_lang::prelude::*;

use crate::{
    execute_sale::*,
    listings::{cancel::*, create::*, update::*},
    offers::{close::*, create::*, update::*},
    redeem_rewards::*,
    reward_center::{create::*, edit::*},
};

declare_id!("rwdLstiU8aJU1DPdoPtocaNKApMhCFdCg283hz8dd3u");

#[program]
pub mod listing_rewards {
    use super::*;

    pub fn create_reward_center(
        ctx: Context<CreateRewardCenter>,
        create_reward_center_params: CreateRewardCenterParams,
    ) -> Result<()> {
        reward_center::create::handler(ctx, create_reward_center_params)
    }

    pub fn edit_reward_center(
        ctx: Context<EditRewardCenter>,
        edit_reward_center_params: EditRewardCenterParams,
    ) -> Result<()> {
        reward_center::edit::handler(ctx, edit_reward_center_params)
    }

    pub fn create_listing(
        ctx: Context<CreateListing>,
        create_listing_params: CreateListingParams,
    ) -> Result<()> {
        listings::create::handler(ctx, create_listing_params)
    }

    pub fn update_listing(
        ctx: Context<UpdateListing>,
        update_listing_params: UpdateListingParams,
    ) -> Result<()> {
        listings::update::handler(ctx, update_listing_params)
    }

    pub fn cancel_listing(
        ctx: Context<CancelListing>,
        cancel_listing_params: CancelListingParams,
    ) -> Result<()> {
        listings::cancel::handler(ctx, cancel_listing_params)
    }

    pub fn create_offer(
        ctx: Context<CreateOffer>,
        create_offer_params: CreateOfferParams,
    ) -> Result<()> {
        offers::create::handler(ctx, create_offer_params)
    }

    pub fn update_offer(
        ctx: Context<UpdateOffer>,
        update_offer_params: UpdateOfferParams,
    ) -> Result<()> {
        offers::update::handler(ctx, update_offer_params)
    }

    pub fn close_offer(
        ctx: Context<CloseOffer>,
        close_offer_params: CloseOfferParams,
    ) -> Result<()> {
        offers::close::handler(ctx, close_offer_params)
    }

    pub fn execute_sale(
        ctx: Context<ExecuteSale>,
        execute_sale_params: ExecuteSaleParams,
    ) -> Result<()> {
        execute_sale::handler(ctx, execute_sale_params)
    }

    pub fn redeem_rewards(ctx: Context<RedemRewards>) -> Result<()> {
        redeem_rewards::redeem_rewards(ctx)
    }
}
