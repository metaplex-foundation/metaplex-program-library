use anchor_lang::{prelude::*, AnchorDeserialize};

use crate::{
    constants::*, errors::*, utils::*, withdraws::withdraw_logic, AuctionHouse, AuthorityScope, *,
};

/// Accounts for the [`auctioneer_withdraw` handler](auction_house/fn.auctioneer_withdraw.html).
#[derive(Accounts, Clone)]
#[instruction(escrow_payment_bump: u8)]
pub struct AuctioneerWithdraw<'info> {
    /// CHECK: Validated in withdraw_logic.
    /// User wallet account.
    pub wallet: UncheckedAccount<'info>,

    /// CHECK: Validated in withdraw_logic.
    /// SPL token account or native SOL account to transfer funds to. If the account is a native SOL account, this is the same as the wallet address.
    #[account(mut)]
    pub receipt_account: UncheckedAccount<'info>,

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

    /// CHECK: Validated in withdraw_logic.
    /// Auction House instance authority account.
    pub authority: UncheckedAccount<'info>,

    /// CHECK: Validated in withdraw_logic.
    /// The auctioneer authority - typically a PDA of the Auctioneer program running this action.
    //#[account(mut)]
    pub auctioneer_authority: Signer<'info>,

    /// Auction House instance PDA account.
    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        bump=auction_house.bump,
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
    pub ata_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

/// Withdraw but with an auctioneer.
pub fn auctioneer_withdraw<'info>(
    ctx: Context<'_, '_, '_, 'info, AuctioneerWithdraw<'info>>,
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
        AuthorityScope::Withdraw,
    )?;

    if escrow_payment_bump
        != *ctx
            .bumps
            .get("escrow_payment_account")
            .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?
    {
        return Err(AuctionHouseError::BumpSeedNotInHashMap.into());
    }

    let mut accounts: Withdraw<'info> = (*ctx.accounts).clone().into();

    withdraw_logic(&mut accounts, escrow_payment_bump, amount)
}
