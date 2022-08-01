use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Mint, Token};
use mpl_auction_house::{AuctionHouse, constants::{AUCTIONEER, PREFIX, FEE_PAYER}, program::AuctionHouse as AuctionHouseProgram, cpi::accounts::AuctioneerCancel};
use crate::{constants::{REWARD_CENTER, REWARDABLE_COLLECTION, LISTING}, state::{RewardCenter, Listing, RewardableCollection}, MetadataAccount, errors::ListingRewardsError, assertions::assert_belongs_to_rewardable_collection};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CancelListingParams {
    pub price: u64,
    pub token_size: u64,
}

#[derive(Accounts, Clone)]
#[instruction(cancel_listing_params: CancelListingParams)]
pub struct CancelListing<'info> {
    /// User wallet account.
    #[account(mut)]
    pub wallet: Signer<'info>,

    /// The Listing Config used for listing settings
    #[account(
        seeds = [
            LISTING.as_bytes(),
            wallet.key().as_ref(),
            metadata.key().as_ref(),
            rewardable_collection.key().as_ref(),
        ],
        bump = listing.bump,
    )]
    pub listing: Account<'info, Listing>,

    /// The collection eligable for rewards
    #[
        account(
            seeds = [
                REWARDABLE_COLLECTION.as_bytes(),
                reward_center.key().as_ref(),
                metadata.collection.as_ref().ok_or(ListingRewardsError::NFTMissingCollection)?.key.as_ref()
        ],
        bump = rewardable_collection.bump
    )]
    pub rewardable_collection: Box<Account<'info, RewardableCollection>>,

    /// Metaplex metadata account decorating SPL mint account.
    pub metadata: Box<Account<'info, MetadataAccount>>,

    /// SPL token account containing the token of the sale to be canceled.
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// Token mint account of SPL token.
    pub token_mint: Box<Account<'info, Mint>>,

    /// CHECK: Validated as a signer in auction_house program cancel_logic.
    /// Auction House instance authority account.
    pub authority: UncheckedAccount<'info>,

    /// The auctioneer program PDA running this auction.
    #[account(
        seeds = [
            REWARD_CENTER.as_bytes(), 
            auction_house.key().as_ref()
        ], 
        bump = reward_center.bump
    )]
    pub reward_center: Box<Account<'info, RewardCenter>>,

    /// Auction House instance PDA account.
    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        seeds::program = auction_house_program,
        bump = auction_house.bump,
        has_one = authority,
        has_one = auction_house_fee_account
    )]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: Validated in cancel_logic.
    /// Auction House instance fee account.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            FEE_PAYER.as_bytes()
        ],
        seeds::program = auction_house_program,
        bump=auction_house.fee_payer_bump
    )]
    pub auction_house_fee_account: UncheckedAccount<'info>,

    /// CHECK: Validated in cancel_logic.
    /// Trade state PDA account representing the bid or ask to be canceled.
    #[account(mut)]
    pub trade_state: UncheckedAccount<'info>,

    /// CHECK: Validated in cancel_logic.
    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(
        seeds = [
            AUCTIONEER.as_bytes(),
            auction_house.key().as_ref(),
            reward_center.key().as_ref()
        ],
        seeds::program = auction_house_program,
        bump = auction_house.auctioneer_pda_bump
    )]
    pub ah_auctioneer_pda: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub auction_house_program: Program<'info, AuctionHouseProgram>,
}

pub fn handler(ctx: Context<CancelListing>, CancelListingParams { price, token_size }: CancelListingParams) -> Result<()> {
    let metadata = &ctx.accounts.metadata;
    let reward_center = &ctx.accounts.reward_center;
    let auction_house = &ctx.accounts.auction_house;
    let rewardable_collection = &ctx.accounts.rewardable_collection;
    let clock = Clock::get()?;
    let auction_house_key = auction_house.key();

    assert_belongs_to_rewardable_collection(metadata, rewardable_collection)?;

    let listing = &mut ctx.accounts.listing;

    listing.canceled_at = Some(clock.unix_timestamp);

    let reward_center_signer_seeds: &[&[&[u8]]] = &[&[
        REWARD_CENTER.as_bytes(),
        auction_house_key.as_ref(),
        &[reward_center.bump],
    ]];

    let cancel_accounts_ctx = CpiContext::new_with_signer(ctx.accounts.auction_house_program.to_account_info(), AuctioneerCancel {
        wallet: ctx.accounts.wallet.to_account_info(),
        token_account: ctx.accounts.token_account.to_account_info(),
        token_mint: ctx.accounts.token_mint.to_account_info(),
        auction_house: ctx.accounts.auction_house.to_account_info(),
        auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
        trade_state: ctx.accounts.trade_state.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        auctioneer_authority: ctx.accounts.reward_center.to_account_info(),
        ah_auctioneer_pda: ctx.accounts.ah_auctioneer_pda.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    }, reward_center_signer_seeds);

    mpl_auction_house::cpi::auctioneer_cancel(cancel_accounts_ctx, price, token_size)?;

    Ok(())
}
