use anchor_lang::{prelude::*, AnchorDeserialize};

use crate::{
    cancel::cancel_logic, constants::*, errors::*, utils::*, AuctionHouse, AuthorityScope, *,
};

/// Accounts for the [`auctioneer_cancel` handler](auction_house/fn.auctioneer_cancel.html).
#[derive(Accounts, Clone)]
#[instruction(buyer_price: u64, token_size: u64)]
pub struct AuctioneerCancel<'info> {
    /// CHECK: Wallet validated as owner in cancel logic.
    /// User wallet account.
    #[account(mut)]
    pub wallet: UncheckedAccount<'info>,

    /// SPL token account containing the token of the sale to be canceled.
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// Token mint account of SPL token.
    pub token_mint: Box<Account<'info, Mint>>,

    /// CHECK: Validated as a signer in cancel_logic.
    /// Auction House instance authority account.
    pub authority: UncheckedAccount<'info>,

    /// CHECK: Validated in ah_auctioneer_pda seeds anbd as a signer in cancel_logic.
    /// The auctioneer authority - typically a PDA of the Auctioneer program running this action.
    pub auctioneer_authority: Signer<'info>,

    /// Auction House instance PDA account.
    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        bump=auction_house.bump,
        has_one=authority,
        has_one=auction_house_fee_account
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
            auctioneer_authority.key().as_ref()
        ],
        bump = ah_auctioneer_pda.bump
    )]
    pub ah_auctioneer_pda: Account<'info, Auctioneer>,

    pub token_program: Program<'info, Token>,
}

pub fn auctioneer_cancel<'info>(
    ctx: Context<'_, '_, '_, 'info, AuctioneerCancel<'info>>,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    let auction_house = &ctx.accounts.auction_house;
    let auctioneer_authority = &ctx.accounts.auctioneer_authority;
    let ah_auctioneer_pda = &ctx.accounts.ah_auctioneer_pda;

    if !auction_house.has_auctioneer {
        return Err(AuctionHouseError::NoAuctioneerProgramSet.into());
    }

    assert_valid_auctioneer_and_scope(
        auction_house,
        &auctioneer_authority.key(),
        ah_auctioneer_pda,
        AuthorityScope::Cancel,
    )?;

    let mut accounts: Cancel<'info> = (*ctx.accounts).clone().into();

    cancel_logic(&mut accounts, buyer_price, token_size)
}
