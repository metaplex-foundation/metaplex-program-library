pub mod assertions;
pub mod constants;
pub mod errors;
pub mod execute_sale;
pub mod listings;
pub mod metaplex_cpi;
pub mod reward_centers;
pub mod offers;
pub mod pda;
pub mod state;

use anchor_lang::prelude::*;

use crate::{
    execute_sale::*,
    listings::{cancel::*, create::*, update::*},
    reward_centers::{create::*, edit::*},
    offers::{close::*, create::*, update::*},
};

declare_id!("rwdLstiU8aJU1DPdoPtocaNKApMhCFdCg283hz8dd3u");

#[program]
pub mod reward_center {
    use super::*;

    pub fn create_reward_center(
        ctx: Context<CreateRewardCenter>,
        create_reward_center_params: CreateRewardCenterParams,
    ) -> Result<()> {
        reward_centers::create::handler(ctx, create_reward_center_params)
    }

    pub fn edit_reward_center(
        ctx: Context<EditRewardCenter>,
        edit_reward_center_params: EditRewardCenterParams,
    ) -> Result<()> {
        reward_centers::edit::handler(ctx, edit_reward_center_params)
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
}
