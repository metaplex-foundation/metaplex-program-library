use anchor_lang::{prelude::*, AnchorDeserialize};

use crate::{constants::*, deposit::deposit_logic, errors::*, AuctionHouse, AuthorityScope, *};

/// Accounts for the [`deposit` handler](auction_house/fn.deposit.html).
#[derive(Accounts)]
#[instruction(escrow_payment_bump: u8)]
pub struct Deposit<'info> {
    /// User wallet account.
    pub wallet: Signer<'info>,

    /// CHECK: Validated in deposit_logic.
    /// User SOL or SPL account to transfer funds from.
    #[account(mut)]
    pub payment_account: UncheckedAccount<'info>,

    /// CHECK: Validated in deposit_logic.
    /// SPL token account transfer authority.
    pub transfer_authority: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Buyer escrow payment account PDA.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            wallet.key().as_ref()
        ],
        bump
    )]
    pub escrow_payment_account: UncheckedAccount<'info>,

    /// Auction House instance treasury mint account.
    pub treasury_mint: Box<Account<'info, Mint>>,

    /// CHECK: Validated in deposit_logic.
    /// Auction House instance authority account.
    pub authority: UncheckedAccount<'info>,

    /// Auction House instance PDA account.
    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        bump=auction_house.bump,
        has_one=authority,
        has_one=treasury_mint,
        has_one=auction_house_fee_account
    )]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
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

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> From<AuctioneerDeposit<'info>> for Deposit<'info> {
    fn from(a: AuctioneerDeposit<'info>) -> Deposit<'info> {
        Deposit {
            wallet: a.wallet,
            payment_account: a.payment_account,
            transfer_authority: a.transfer_authority,
            escrow_payment_account: a.escrow_payment_account,
            treasury_mint: a.treasury_mint,
            authority: a.authority,
            auction_house: a.auction_house,
            auction_house_fee_account: a.auction_house_fee_account,
            token_program: a.token_program,
            system_program: a.system_program,
            rent: a.rent,
        }
    }
}

pub fn deposit<'info>(
    ctx: Context<'_, '_, '_, 'info, Deposit<'info>>,
    escrow_payment_bump: u8,
    amount: u64,
) -> Result<()> {
    let auction_house = &ctx.accounts.auction_house;

    // If it has an auctioneer authority delegated must use auctioneer_* handler.
    if auction_house.has_auctioneer && auction_house.scopes[AuthorityScope::Deposit as usize] {
        return Err(AuctionHouseError::MustUseAuctioneerHandler.into());
    }

    if escrow_payment_bump
        != *ctx
            .bumps
            .get("escrow_payment_account")
            .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?
    {
        return Err(AuctionHouseError::BumpSeedNotInHashMap.into());
    }

    deposit_logic(ctx.accounts, escrow_payment_bump, amount)
}
