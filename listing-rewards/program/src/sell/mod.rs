use anchor_lang::{context::Context, prelude::*, AnchorDeserialize};
use anchor_spl::token::{Token, TokenAccount};

use crate::{
    constants::{LISTING, REWARDABLE_COLLECTION, REWARD_CENTER},
    reward_center::RewardCenter,
    rewardable_collection::RewardableCollection,
};
use mpl_auction_house::{
    constants::{AUCTIONEER, FEE_PAYER, PREFIX, SIGNER},
    cpi::accounts::AuctioneerSell,
    program::AuctionHouse as AuctionHouseProgram,
    AuctionHouse,
};

#[account]
pub struct Listing {
    pub trade_state: Pubkey,
    pub reward_center: Pubkey,
    pub seller: Pubkey,
    pub metadata: Pubkey,
    pub price: u64,
    pub token_size: u64,
    pub bump: u8,
    pub trade_state_bump: u8,
    pub created_at: i64,
    pub canceled_at: Option<i64>,
    pub purchased_at: Option<i64>,
    pub rewardable_collection: Pubkey,
    pub reward_redeemed_at: Option<i64>,
}

impl Listing {
    pub fn size() -> usize {
        8 + // deliminator
        32 + // trade_state
        32 + // reward_center
        32 + // seller
        32 + // metadata
        8 + // price
        8 + // token_size
        1 + // bump
        1 + // trade_state_bump
        8 + // created_at
        1 + 8 + // canceled_at
        1 + 8 + // canceled_at
        32 + // rewardable_collection
        1 + 8 // reward_redeemed_at
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SellParams {
    pub price: u64,
    pub token_size: u64,
    pub collection: Pubkey,
    pub trade_state_bump: u8,
    pub free_trade_state_bump: u8,
    pub program_as_signer_bump: u8,
}

/// Accounts for the [`sell` handler](listing_rewards/fn.sell.html).
#[derive(Accounts, Clone)]
#[instruction(sell_params: SellParams)]
pub struct Sell<'info> {
    /// Auction House Program used for CPI call
    pub auction_house_program: Program<'info, AuctionHouseProgram>,

    // Accounts used for Auctioneer
    /// The Listing Config used for listing settings
    #[account(
        init,
        payer=wallet,
        space=Listing::size(),
        seeds=[
            LISTING.as_bytes(),
            wallet.key().as_ref(),
            token_account.mint.as_ref(),
            rewardable_collection.key().as_ref(),
        ],
        bump,
    )]
    pub listing: Account<'info, Listing>,

    /// The auctioneer program PDA running this auction.
    #[account(seeds = [REWARD_CENTER.as_bytes(), auction_house.key().as_ref()], bump = reward_center.bump)]
    pub reward_center: Box<Account<'info, RewardCenter>>,

    /// The collection eligable for rewards
    #[account(seeds = [REWARDABLE_COLLECTION.as_bytes(), reward_center.key().as_ref(), sell_params.collection.as_ref()], bump)]
    pub rewardable_collection: Box<Account<'info, RewardableCollection>>,

    // Accounts passed into Auction House CPI call
    /// CHECK: Verified through CPI
    /// User wallet account.
    #[account(mut)]
    pub wallet: UncheckedAccount<'info>,

    /// SPL token account containing token for sale.
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: Verified through CPI
    /// Metaplex metadata account decorating SPL mint account.
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Auction House authority account.
    pub authority: UncheckedAccount<'info>,

    /// Auction House instance PDA account.
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], seeds::program=auction_house_program, bump=auction_house.bump, has_one=auction_house_fee_account)]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Auction House instance fee account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], seeds::program=auction_house_program, bump=auction_house.fee_payer_bump)]
    pub auction_house_fee_account: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Seller trade state PDA account encoding the sell order.
    #[account(mut, seeds=[PREFIX.as_bytes(), wallet.key().as_ref(), auction_house.key().as_ref(), token_account.key().as_ref(), auction_house.treasury_mint.as_ref(), token_account.mint.as_ref(),  &u64::MAX.to_le_bytes(), &sell_params.token_size.to_le_bytes()], seeds::program=auction_house_program, bump=sell_params.trade_state_bump)]
    pub seller_trade_state: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Free seller trade state PDA account encoding a free sell order.
    #[account(mut, seeds=[PREFIX.as_bytes(), wallet.key().as_ref(), auction_house.key().as_ref(), token_account.key().as_ref(), auction_house.treasury_mint.as_ref(), token_account.mint.as_ref(), &0u64.to_le_bytes(), &sell_params.token_size.to_le_bytes()], seeds::program=auction_house_program, bump=sell_params.free_trade_state_bump)]
    pub free_seller_trade_state: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(seeds = [AUCTIONEER.as_bytes(), auction_house.key().as_ref(), reward_center.key().as_ref()], seeds::program=auction_house_program, bump = auction_house.auctioneer_pda_bump)]
    pub ah_auctioneer_pda: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(seeds=[PREFIX.as_bytes(), SIGNER.as_bytes()], seeds::program=auction_house_program, bump=sell_params.program_as_signer_bump)]
    pub program_as_signer: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn sell(
    ctx: Context<Sell>,
    SellParams {
        token_size,
        trade_state_bump,
        free_trade_state_bump,
        program_as_signer_bump,
        ..
    }: SellParams,
) -> Result<()> {
    let reward_center = &ctx.accounts.reward_center;
    let auction_house = &ctx.accounts.auction_house;
    let auction_house_key = auction_house.key();

    let seeds = &[
        REWARD_CENTER.as_bytes(),
        auction_house_key.as_ref(),
        &[reward_center.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    mpl_auction_house::cpi::auctioneer_sell(
        ctx.accounts
            .set_auctioneer_sell_ctx()
            .with_signer(signer_seeds),
        trade_state_bump,
        free_trade_state_bump,
        program_as_signer_bump,
        token_size,
    )?;

    Ok(())
}

impl<'info> Sell<'info> {
    pub fn set_auctioneer_sell_ctx(&self) -> CpiContext<'_, '_, '_, 'info, AuctioneerSell<'info>> {
        let cpi_program = self.auction_house_program.to_account_info();
        let accounts = (&*self).into();

        CpiContext::new(cpi_program, accounts)
    }
}

impl<'info> From<&Sell<'info>> for AuctioneerSell<'info> {
    fn from(accounts: &Sell<'info>) -> Self {
        Self {
            wallet: accounts.wallet.to_account_info().clone(),
            token_account: accounts.token_account.to_account_info(),
            metadata: accounts.metadata.to_account_info(),
            auction_house: accounts.auction_house.to_account_info(),
            auction_house_fee_account: accounts.auction_house_fee_account.to_account_info(),
            seller_trade_state: accounts.seller_trade_state.to_account_info(),
            free_seller_trade_state: accounts.free_seller_trade_state.to_account_info(),
            authority: accounts.authority.to_account_info(),
            auctioneer_authority: accounts.reward_center.to_account_info(),
            ah_auctioneer_pda: accounts.ah_auctioneer_pda.to_account_info(),
            token_program: accounts.token_program.to_account_info(),
            system_program: accounts.system_program.to_account_info(),
            program_as_signer: accounts.program_as_signer.to_account_info(),
            rent: accounts.rent.to_account_info(),
        }
    }
}
