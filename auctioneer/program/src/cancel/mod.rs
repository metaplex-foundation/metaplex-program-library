use anchor_lang::{prelude::*, AnchorDeserialize};
use anchor_spl::token::{Mint, Token, TokenAccount};

use mpl_auction_house::{
    self,
    constants::{AUCTIONEER, FEE_PAYER, PREFIX},
    //auction_house::{
    cpi::accounts::CancelWithAuctioneer as AHCancel,
    program::AuctionHouse as AuctionHouseProgram, //program::auction_house as AuctionHouseProgram,
    //program::auction_house,
    //},
    AuctionHouse,
};

/// Accounts for the [`cancel` handler](auction_house/fn.cancel.html).
#[derive(Accounts, Clone)]
#[instruction(buyer_price: u64, token_size: u64)]
pub struct AuctioneerCancel<'info> {
    /// Auction House Program
    pub auction_house_program: Program<'info, AuctionHouseProgram>,

    /// CHECK: TODO
    /// User wallet account.
    #[account(mut)]
    pub wallet: UncheckedAccount<'info>,

    /// SPL token account containing the token of the sale to be canceled.
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// Token mint account of SPL token.
    pub token_mint: Box<Account<'info, Mint>>,

    /// CHECK: TODO
    /// Auction House instance authority account.
    pub authority: UncheckedAccount<'info>,

    /// Auction House instance PDA account.
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], bump=auction_house.bump, has_one=authority, has_one=auction_house_fee_account)]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: TODO
    /// Auction House instance fee account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], bump=auction_house.fee_payer_bump)]
    pub auction_house_fee_account: UncheckedAccount<'info>,

    /// CHECK: TODO
    /// Trade state PDA account representing the bid or ask to be canceled.
    #[account(mut)]
    pub trade_state: UncheckedAccount<'info>,

    /// CHECK: TODO
    /// The auctioneer program PDA running this auction.
    pub auctioneer_authority: UncheckedAccount<'info>,

    /// CHECK: TODO
    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(seeds = [AUCTIONEER.as_bytes(), auction_house.key().as_ref(), auctioneer_authority.key().as_ref()], bump = auction_house.auctioneer_pda_bump)]
    pub ah_auctioneer_pda: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

// Cancel a bid or ask by revoking the token delegate, transferring all lamports from the trade state account to the fee payer, and setting the trade state account data to zero so it can be garbage collected.

pub fn auctioneer_cancel<'info>(
    ctx: Context<'_, '_, '_, 'info, AuctioneerCancel<'info>>,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    let cpi_program = ctx.accounts.auction_house_program.to_account_info();
    let cpi_accounts = AHCancel {
        wallet: ctx.accounts.wallet.to_account_info(),
        token_account: ctx.accounts.token_account.to_account_info(),
        token_mint: ctx.accounts.token_mint.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        auction_house: ctx.accounts.auction_house.to_account_info(),
        auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
        trade_state: ctx.accounts.trade_state.to_account_info(),
        auctioneer_authority: ctx.accounts.auctioneer_authority.to_account_info(),
        ah_auctioneer_pda: ctx.accounts.ah_auctioneer_pda.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    mpl_auction_house::cpi::cancel_with_auctioneer(cpi_ctx, buyer_price, token_size)
}
