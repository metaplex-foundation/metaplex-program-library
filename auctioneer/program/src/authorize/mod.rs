use anchor_lang::{prelude::*, AnchorDeserialize};
use anchor_spl::token::{Mint, Token};

use crate::{constants::*, errors::*};

use mpl_auction_house::{
    self,
    constants::{AUCTIONEER, FEE_PAYER, PREFIX, SIGNER},
    //auction_house::{
    cpi::accounts::AuctioneerSell as AHSell,
    program::AuctionHouse as AuctionHouseProgram, //program::auction_house as AuctionHouseProgram,
    //program::auction_house,
    //},
    AuctionHouse,
};

/// Accounts for the [`deposit` handler](auction_house/fn.deposit.html).
#[derive(Accounts, Clone)]
//#[instruction()]
pub struct AuctioneerAuthorize<'info> {
    /// User wallet account.
    #[account(mut)]
    pub wallet: Signer<'info>,

    /// Auction House instance PDA account.
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], seeds::program=mpl_auction_house::id(), bump=auction_house.bump)]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// The auctioneer program PDA running this auction.
    #[account(init, payer=wallet, space = 8 + 1, seeds = [AUCTIONEER.as_bytes(), auction_house.key().as_ref()], bump)]
    pub auctioneer_authority: Account<'info, AuctioneerAuthority>,

    pub system_program: Program<'info, System>,
}

pub fn auctioneer_authorize<'info>(
    ctx: Context<'_, '_, '_, 'info, AuctioneerAuthorize<'info>>,
) -> Result<()> {
    if ctx.accounts.wallet.key() != ctx.accounts.auction_house.authority {
        return err!(AuctioneerError::SignerNotAuth);
    }

    ctx.accounts.auctioneer_authority.bump = *ctx
        .bumps
        .get("auctioneer_authority")
        .ok_or(AuctioneerError::BumpSeedNotInHashMap)?;

    Ok(())
}

#[account]
pub struct AuctioneerAuthority {
    pub bump: u8,
}
