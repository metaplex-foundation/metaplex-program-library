use anchor_lang::{prelude::*, AnchorDeserialize};
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    bids::auctioneer_bid_logic::auctioneer_bid_logic, constants::*, errors::AuctionHouseError,
    AuctionHouse, Auctioneer,
};

/// Accounts for the [`auctioneer_public_bid` handler](fn.auctioneer_public_bid.html).
#[derive(Accounts)]
#[instruction(
    trade_state_bump: u8,
    escrow_payment_bump: u8,
    buyer_price: u64,
    token_size: u64
)]
pub struct AuctioneerPublicBuy<'info> {
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

    /// CHECK: Verified in ah_auctioneer_pda seeds and in bid logic.
    /// The auctioneer authority - typically a PDA of the Auctioneer program running this action.
    auctioneer_authority: Signer<'info>,

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

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(
        seeds = [
            AUCTIONEER.as_bytes(),
            auction_house.key().as_ref(),
            auctioneer_authority.key().as_ref()
        ],
        bump = ah_auctioneer_pda.bump
    )]
    pub ah_auctioneer_pda: Account<'info, Auctioneer>,

    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

/// Create a bid on a specific SPL token.
/// Public bids are specific to the token itself, rather than the auction, and remain open indefinitely until either the user closes it or the requirements for the bid are met and it is matched with a counter bid and closed as a transaction.
pub fn auctioneer_public_bid(
    ctx: Context<AuctioneerPublicBuy>,
    trade_state_bump: u8,
    escrow_payment_bump: u8,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    auctioneer_bid_logic(
        ctx.accounts.wallet.to_owned(),
        ctx.accounts.payment_account.to_owned(),
        ctx.accounts.transfer_authority.to_owned(),
        *ctx.accounts.treasury_mint.to_owned(),
        *ctx.accounts.token_account.to_owned(),
        ctx.accounts.metadata.to_owned(),
        ctx.accounts.escrow_payment_account.to_owned(),
        &mut ctx.accounts.auction_house,
        ctx.accounts.auction_house_fee_account.to_owned(),
        ctx.accounts.buyer_trade_state.to_owned(),
        ctx.accounts.authority.to_owned(),
        ctx.accounts.auctioneer_authority.to_owned(),
        ctx.accounts.ah_auctioneer_pda.to_owned(),
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
