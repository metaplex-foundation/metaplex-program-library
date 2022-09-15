use anchor_lang::{prelude::*, AnchorDeserialize};

use mpl_auction_house::{constants::PREFIX, AuctionHouse};

use crate::{
    constants::REWARD_CENTER,
    errors::ListingRewardsError,
    state::listing_rewards::{ListingRewardRules, RewardCenter},
};

/// Options to set on the reward center
#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct EditRewardCenterParams {
    pub listing_reward_rules: ListingRewardRules,
}

/// Accounts for the [`create_reward_center` handler](listing_rewards/fn.create_reward_center.html).
#[derive(Accounts, Clone)]
#[instruction(create_reward_center_args: EditRewardCenterParams)]
pub struct EditRewardCenter<'info> {
    /// User wallet account.
    #[
      account(
        mut,
        constraint = wallet.key() == auction_house.authority @ ListingRewardsError::SignerNotAuthorized
      )
    ]
    pub wallet: Signer<'info>,

    /// Auction House instance PDA account.
    #[account(
        seeds = [
            PREFIX.as_bytes(), 
            auction_house.creator.as_ref(), 
            auction_house.treasury_mint.as_ref()
        ], 
        seeds::program = mpl_auction_house::id(), 
        bump = auction_house.bump
    )]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// The auctioneer program PDA running this auction.
    #[account(
        mut,
        seeds = [REWARD_CENTER.as_bytes(), auction_house.key().as_ref()], 
        bump
    )]
    pub reward_center: Account<'info, RewardCenter>,
}

pub fn handler(
    ctx: Context<EditRewardCenter>,
    reward_center_params: EditRewardCenterParams,
) -> Result<()> {
    let reward_center = &mut ctx.accounts.reward_center;
    reward_center.listing_reward_rules = reward_center_params.listing_reward_rules;

    Ok(())
}
