pub mod assertions;
pub mod constants;
pub mod errors;
pub mod offers;
pub mod pda;
pub mod redeem_rewards;
pub mod reward_center;
pub mod rewardable_collection;
pub mod sell;
pub mod state;

use anchor_lang::prelude::*;
use core::ops::Deref;

use crate::{
    offers::{close::*, create::*},
    redeem_rewards::*,
    reward_center::*,
    rewardable_collection::*,
    sell::*,
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
        offers::create::handler(ctx, create_offer_params)
    }

    pub fn close_offer(
        ctx: Context<CloseOffer>,
        close_offer_params: CloseOfferParams,
    ) -> Result<()> {
        offers::close::handler(ctx, close_offer_params)
    }

    pub fn redeem_rewards(ctx: Context<RedemRewards>) -> Result<()> {
        redeem_rewards::redeem_rewards(ctx)
    }
}
