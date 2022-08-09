use anchor_lang::{prelude::*, AnchorDeserialize};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use mpl_auction_house::{constants::PREFIX, AuctionHouse};

use crate::{
    constants::REWARD_CENTER,
    errors::ListingRewardsError,
    state::{ListingRewardRules, RewardCenter},
};

/// Options to set on the reward center
#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct CreateRewardCenterParams {
    pub collection_oracle: Option<Pubkey>,
    pub listing_reward_rules: ListingRewardRules,
}

/// Accounts for the [`create_reward_center` handler](listing_rewards/fn.create_reward_center.html).
#[derive(Accounts, Clone)]
#[instruction(create_reward_center_args: CreateRewardCenterParams)]
pub struct CreateRewardCenter<'info> {
    /// User wallet account.
    #[
      account(
        mut,
        constraint = wallet.key() == auction_house.authority @ ListingRewardsError::SignerNotAuthorized
      )
    ]
    pub wallet: Signer<'info>,

    /// the mint of the token to use as rewards.
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = wallet,
        associated_token::mint = mint,
        associated_token::authority = reward_center
    )]
    pub associated_token_account: Account<'info, TokenAccount>,

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
        init, 
        payer = wallet, 
        space = RewardCenter::size(), 
        seeds = [REWARD_CENTER.as_bytes(), auction_house.key().as_ref()], 
        bump
    )]
    pub reward_center: Account<'info, RewardCenter>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<CreateRewardCenter>,
    reward_center_params: CreateRewardCenterParams,
) -> Result<()> {
    let mint = &ctx.accounts.mint;
    let auction_house = &ctx.accounts.auction_house;
    let reward_center = &mut ctx.accounts.reward_center;

    reward_center.token_mint = mint.key();
    reward_center.auction_house = auction_house.key();
    reward_center.collection_oracle = reward_center_params.collection_oracle;
    reward_center.listing_reward_rules = reward_center_params.listing_reward_rules;
    reward_center.bump = *ctx
        .bumps
        .get(REWARD_CENTER)
        .ok_or(ListingRewardsError::BumpSeedNotInHashMap)?;

    Ok(())
}
