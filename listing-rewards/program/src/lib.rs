pub mod assertions;
pub mod constants;
pub mod errors;
pub mod execute_sale;
pub mod listings;
pub mod offers;
pub mod pda;
pub mod redeem_rewards;
pub mod reward_center;
pub mod rewardable_collection;
pub mod state;

use anchor_lang::prelude::*;
use core::ops::Deref;

use crate::{
    execute_sale::*,
    listings::{cancel::*, create::*},
    offers::{close::*, create::*},
    redeem_rewards::*,
    reward_center::{create::*, edit::*},
    rewardable_collection::{create::*, delete::*},
};

// TODO: Remove when added to Anchor https://github.com/coral-xyz/anchor/pull/2014
#[derive(Clone, Debug, PartialEq)]
pub struct MetadataAccount(mpl_token_metadata::state::Metadata);

impl MetadataAccount {
    pub const LEN: usize = mpl_token_metadata::state::MAX_METADATA_LEN;
}

impl anchor_lang::AccountDeserialize for MetadataAccount {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        let result = mpl_token_metadata::state::Metadata::deserialize(buf)?;
        return Ok(MetadataAccount(result));
    }
}

impl anchor_lang::AccountSerialize for MetadataAccount {}

impl anchor_lang::Owner for MetadataAccount {
    fn owner() -> Pubkey {
        mpl_token_metadata::ID
    }
}

impl Deref for MetadataAccount {
    type Target = mpl_token_metadata::state::Metadata;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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

    pub fn create_rewardable_collection(
        ctx: Context<CreateRewardableCollection>,
        rewardable_collection_params: CreateRewardableCollectionParams,
    ) -> Result<()> {
        rewardable_collection::create::handler(ctx, rewardable_collection_params)
    }

    pub fn delete_rewardable_collection(
        ctx: Context<DeleteRewardableCollection>,
        rewardable_collection_params: DeleteRewardableCollectionParams,
    ) -> Result<()> {
        rewardable_collection::delete::handler(ctx, rewardable_collection_params)
    }

    pub fn create_listing(
        ctx: Context<CreateListing>,
        sell_params: CreateListingParams,
    ) -> Result<()> {
        listings::create::handler(ctx, sell_params)
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
