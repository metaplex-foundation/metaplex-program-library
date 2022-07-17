use anchor_lang::{context::Context, prelude::*};
use anchor_spl::{token::{Token, TokenAccount, Mint}, associated_token::AssociatedToken};

use crate::{
    MetadataAccount,
    constants::{LISTING, REWARDABLE_COLLECTION, REWARD_CENTER},
    rewardable_collection::RewardableCollection,
    reward_center::RewardCenter,
    sell::Listing,
    errors::ListingRewardsError,
    assertions::assert_listing_reward_redemption_eligibility
};

/// Accounts for the [`redeem_rewards` handler](listing_rewards/fn.redeem_rewards.html).
#[derive(Accounts, Clone)]
pub struct RedemRewards<'info> {
    // Accounts used for Auctioneer
    /// The Listing Config used for listing settings
    #[account(
        seeds=[
            LISTING.as_bytes(),
            wallet.key().as_ref(),
            metadata.key().as_ref(),
            rewardable_collection.key().as_ref(),
        ],
        bump = listing.bump,
    )]
    pub listing: Account<'info, Listing>,

    #[account(
        mut,
        constraint = listing.seller == wallet.key() @ ListingRewardsError::SellerWalletMismatch
    )]
    pub wallet: Signer<'info>,

    #[
        account(
            mut,
            constraint = reward_center_associated_token_account.mint == mint.key()
        )
    ]
    pub reward_center_associated_token_account: Account<'info, TokenAccount>,

    pub metadata: Account<'info, MetadataAccount>,

    #[account(
      init_if_needed,
      payer = wallet,
      associated_token::mint = mint,
      associated_token::authority = wallet
    )]
    pub wallet_associated_token_account: Account<'info, TokenAccount>,

    #[account(address = reward_center.token_mint)]
    pub mint: Account<'info, Mint>,

    /// The auctioneer program PDA running this auction.
    #[account(seeds = [REWARD_CENTER.as_bytes(), reward_center.auction_house.as_ref()], bump = reward_center.bump)]
    pub reward_center: Account<'info, RewardCenter>,

    /// The collection eligable for rewards
    #[account(seeds = [REWARDABLE_COLLECTION.as_bytes(), reward_center.key().as_ref(), rewardable_collection.collection.as_ref()], bump = rewardable_collection.bump)]
    pub rewardable_collection: Account<'info, RewardableCollection>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn redeem_rewards(ctx: Context<RedemRewards>) -> Result<()> {
    let listing = &ctx.accounts.listing;
    let rewardable_collection = &ctx.accounts.rewardable_collection;
    let reward_center = &ctx.accounts.reward_center;

    assert_listing_reward_redemption_eligibility(listing, reward_center)?;

        
    Ok(())
}
