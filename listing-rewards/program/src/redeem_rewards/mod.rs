use anchor_lang::{context::Context, prelude::*};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{
    assertions::assert_listing_reward_redemption_eligibility,
    constants::{LISTING, REWARD_CENTER},
    errors::ListingRewardsError,
    state::{
        listing_rewards::{Listing, RewardCenter},
        metaplex_anchor::TokenMetadata,
    },
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
            reward_center.key().as_ref(),
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

    pub metadata: Account<'info, TokenMetadata>,

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

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn redeem_rewards(ctx: Context<RedemRewards>) -> Result<()> {
    let listing = &ctx.accounts.listing;
    let reward_center = &ctx.accounts.reward_center;

    assert_listing_reward_redemption_eligibility(listing, reward_center)?;

    Ok(())
}
