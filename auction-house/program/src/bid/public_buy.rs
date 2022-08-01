use anchor_lang::{prelude::*, AnchorDeserialize};
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{bid::bid_logic::bid_logic, constants::*, errors::AuctionHouseError, AuctionHouse};

/// Accounts for the [`public_bid` handler](fn.public_bid.html).
#[derive(Accounts)]
#[instruction(
    trade_state_bump: u8,
    escrow_payment_bump: u8,
    buyer_price: u64,
    token_size: u64
)]
pub struct PublicBuy<'info> {
    wallet: Signer<'info>,

    /// CHECK: Validated in public_bid_logic.
    #[account(mut)]
    payment_account: UncheckedAccount<'info>,

    /// CHECK: Validated in public_bid_logic.
    transfer_authority: UncheckedAccount<'info>,

    treasury_mint: Box<Account<'info, Mint>>,
    token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: Validated in public_bid_logic.
    metadata: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            wallet.key().as_ref()
        ],
        bump
    )]
    escrow_payment_account: UncheckedAccount<'info>,

    /// CHECK: Verified with has_one constraint on auction house account.
    authority: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        bump = auction_house.bump,
        has_one = authority,
        has_one = treasury_mint,
        has_one = auction_house_fee_account
    )]
    auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            FEE_PAYER.as_bytes()
        ],
        bump = auction_house.fee_payer_bump
    )]
    auction_house_fee_account: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            wallet.key().as_ref(),
            auction_house.key().as_ref(),
            treasury_mint.key().as_ref(),
            token_account.mint.as_ref(),
            buyer_price.to_le_bytes().as_ref(),
            token_size.to_le_bytes().as_ref()
        ],
        bump
    )]
    buyer_trade_state: UncheckedAccount<'info>,

    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

/// Create a bid on a specific SPL token.
/// Public bids are specific to the token itself, rather than the auction, and remain open indefinitely until either the user closes it or the requirements for the bid are met and it is matched with a counter bid and closed as a transaction.
pub fn public_bid(
    ctx: Context<PublicBuy>,
    trade_state_bump: u8,
    escrow_payment_bump: u8,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    bid_logic(
        ctx.accounts.wallet.to_owned(),
        ctx.accounts.payment_account.to_owned(),
        ctx.accounts.transfer_authority.to_owned(),
        *ctx.accounts.treasury_mint.to_owned(),
        *ctx.accounts.token_account.to_owned(),
        ctx.accounts.metadata.to_owned(),
        ctx.accounts.escrow_payment_account.to_owned(),
        ctx.accounts.authority.to_owned(),
        *ctx.accounts.auction_house.to_owned(),
        ctx.accounts.auction_house_fee_account.to_owned(),
        ctx.accounts.buyer_trade_state.to_owned(),
        ctx.accounts.token_program.to_owned(),
        ctx.accounts.system_program.to_owned(),
        ctx.accounts.rent.to_owned(),
        trade_state_bump,
        escrow_payment_bump,
        buyer_price,
        token_size,
        true,
        *ctx.bumps
            .get("escrow_payment_account")
            .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?,
        *ctx.bumps
            .get("buyer_trade_state")
            .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?,
    )
}
