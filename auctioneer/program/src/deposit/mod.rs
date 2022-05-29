use anchor_lang::{prelude::*, AnchorDeserialize};
use anchor_spl::token::{Mint, Token};

use mpl_auction_house::{
    self,
    constants::{AUCTIONEER, FEE_PAYER, PREFIX},
    cpi::accounts::AuctioneerDeposit as AHDeposit,
    program::AuctionHouse as AuctionHouseProgram,
    AuctionHouse,
};

/// Accounts for the [`deposit` handler](auction_house/fn.deposit.html).
#[derive(Accounts, Clone)]
#[instruction(escrow_payment_bump: u8, auctioneer_authority_bump: u8)]
pub struct AuctioneerDeposit<'info> {
    /// Auction House Program
    pub auction_house_program: Program<'info, AuctionHouseProgram>,

    /// User wallet account.
    pub wallet: Signer<'info>,

    /// CHECK: Verified through CPI
    /// User SOL or SPL account to transfer funds from.
    #[account(mut)]
    pub payment_account: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// SPL token account transfer authority.
    pub transfer_authority: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Buyer escrow payment account PDA.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), wallet.key().as_ref()], seeds::program=auction_house_program, bump=escrow_payment_bump)]
    pub escrow_payment_account: UncheckedAccount<'info>,

    /// Auction House instance treasury mint account.
    pub treasury_mint: Box<Account<'info, Mint>>,

    /// CHECK: Verified through CPI
    /// Auction House instance authority account.
    pub authority: UncheckedAccount<'info>,

    /// Auction House instance PDA account.
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], seeds::program=auction_house_program, bump=auction_house.bump, has_one=authority, has_one=treasury_mint, has_one=auction_house_fee_account)]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Auction House instance fee account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], seeds::program=auction_house_program, bump=auction_house.fee_payer_bump)]
    pub auction_house_fee_account: UncheckedAccount<'info>,

    /// CHECK: Validated in deposit_logic.
    /// The auctioneer program PDA running this auction.
    pub auctioneer_authority: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(seeds = [AUCTIONEER.as_bytes(), auction_house.key().as_ref(), auctioneer_authority.key().as_ref()], seeds::program=auction_house_program, bump = auction_house.auctioneer_pda_bump)]
    pub ah_auctioneer_pda: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn auctioneer_deposit<'info>(
    ctx: Context<'_, '_, '_, 'info, AuctioneerDeposit<'info>>,
    escrow_payment_bump: u8,
    auctioneer_authority_bump: u8,
    amount: u64,
) -> Result<()> {
    let cpi_program = ctx.accounts.auction_house_program.to_account_info();
    let cpi_accounts = AHDeposit {
        wallet: ctx.accounts.wallet.to_account_info(),
        payment_account: ctx.accounts.payment_account.to_account_info(),
        transfer_authority: ctx.accounts.transfer_authority.to_account_info(),
        escrow_payment_account: ctx.accounts.escrow_payment_account.to_account_info(),
        treasury_mint: ctx.accounts.treasury_mint.to_account_info(),
        auction_house: ctx.accounts.auction_house.to_account_info(),
        auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        auctioneer_authority: ctx.accounts.auctioneer_authority.to_account_info(),
        ah_auctioneer_pda: ctx.accounts.ah_auctioneer_pda.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };

    let auction_house = &ctx.accounts.auction_house;
    let ah_key = auction_house.key();
    let auctioneer_authority = &ctx.accounts.auctioneer_authority;
    let _aa_key = auctioneer_authority.key();

    let auctioneer_seeds = [
        AUCTIONEER.as_bytes(),
        ah_key.as_ref(),
        &[auctioneer_authority_bump],
    ];

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    mpl_auction_house::cpi::auctioneer_deposit(
        cpi_ctx.with_signer(&[&auctioneer_seeds]),
        escrow_payment_bump,
        amount,
    )
}
