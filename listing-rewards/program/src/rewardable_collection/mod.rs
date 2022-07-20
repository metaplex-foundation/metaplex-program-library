use anchor_lang::{prelude::*, AnchorDeserialize};
use mpl_auction_house::{self, constants::PREFIX, AuctionHouse};

use crate::{
    assertions::*,
    constants::{REWARDABLE_COLLECTION, REWARD_CENTER},
    errors::ListingRewardsError,
    reward_center::*,
};

#[account]
pub struct RewardableCollection {
    /// the mint address of the collection
    pub collection: Pubkey,
    /// the address of the associated reward center
    pub reward_center: Pubkey,
    /// the pda bump
    pub bump: u8,
}

impl RewardableCollection {
    pub fn size() -> usize {
        8 + // deliminator
      32 + // collection
      32 + // reward_center
      1 // pda bump
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct CreateRewardableCollectionParams {
    pub collection: Pubkey,
}

/// Accounts for the [`create_rewardable_collection` handler](listing_rewards/fn.create_rewardable_collection.html).
#[derive(Accounts, Clone)]
#[instruction(rewardable_collection_params: CreateRewardableCollectionParams)]
pub struct CreateRewardableCollection<'info> {
    /// The wallet of collection maintainer. Either the auction house authority or collection oracle.
    #[account(mut)]
    pub wallet: Signer<'info>,

    /// The auctioneer program PDA running this auction.
    #[account(init, payer = wallet, space = RewardableCollection::size(), seeds = [REWARDABLE_COLLECTION.as_bytes(), reward_center.key().as_ref(), rewardable_collection_params.collection.as_ref()], bump)]
    pub rewardable_collection: Account<'info, RewardableCollection>,

    /// Auction House instance PDA account.
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], seeds::program=mpl_auction_house::id(), bump=auction_house.bump)]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// The auctioneer program PDA running this auction.
    #[account(seeds = [REWARD_CENTER.as_bytes(), auction_house.key().as_ref()], bump = reward_center.bump)]
    pub reward_center: Account<'info, RewardCenter>,

    pub system_program: Program<'info, System>,
}

pub fn create_rewardable_collection(
    ctx: Context<CreateRewardableCollection>,
    CreateRewardableCollectionParams { collection }: CreateRewardableCollectionParams,
) -> Result<()> {
    let rewardable_collection = &mut ctx.accounts.rewardable_collection;
    let reward_center = &ctx.accounts.reward_center;
    let auction_house = &ctx.accounts.auction_house;
    let wallet = &ctx.accounts.wallet;

    assert_rewardable_collection_maintainer(wallet.key(), auction_house, reward_center)?;

    rewardable_collection.collection = collection;
    rewardable_collection.reward_center = reward_center.key();
    rewardable_collection.bump = *ctx
        .bumps
        .get(REWARDABLE_COLLECTION)
        .ok_or(ListingRewardsError::BumpSeedNotInHashMap)?;

    Ok(())
}
