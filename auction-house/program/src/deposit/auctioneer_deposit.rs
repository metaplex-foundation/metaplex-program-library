use anchor_lang::{prelude::*, AnchorDeserialize};

use crate::{
    constants::*, deposit::deposit_logic, errors::*, utils::*, AuctionHouse, AuthorityScope, *,
};

/// Accounts for the [`deposit` handler](auction_house/fn.deposit.html).
#[derive(Accounts, Clone)]
#[instruction(escrow_payment_bump: u8)]
pub struct AuctioneerDeposit<'info> {
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

    /// CHECK: Validated in ah_auctioneer_pda seeds and deposit_logic.
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

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn auctioneer_deposit<'info>(
    ctx: Context<'_, '_, '_, 'info, AuctioneerDeposit<'info>>,
    escrow_payment_bump: u8,
    amount: u64,
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
        AuthorityScope::Deposit,
    )?;

    if escrow_payment_bump
        != *ctx
            .bumps
            .get("escrow_payment_account")
            .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?
    {
        return Err(AuctionHouseError::BumpSeedNotInHashMap.into());
    }

    let mut accounts: Deposit<'info> = (*ctx.accounts).clone().into();

    deposit_logic(&mut accounts, escrow_payment_bump, amount)
}
