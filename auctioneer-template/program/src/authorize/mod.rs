use anchor_lang::{prelude::*, AnchorDeserialize};

use crate::errors::*;

use mpl_auction_house::{
    self,
    constants::{AUCTIONEER, PREFIX},
    AuctionHouse,
};

/// Accounts for the [`auctioneer_authorize` handler](auction_house/fn.auctioneer_authorize.html).
#[derive(Accounts, Clone)]
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
