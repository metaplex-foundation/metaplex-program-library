use anchor_lang::prelude::*;

use crate::{constants::*, errors::AuctionHouseError, AuctionHouse, Auctioneer, AuthorityScope};

/// Accounts for the [`delegate_auctioneer` handler](auction_house/fn.delegate_auctioneer.html).
#[derive(Accounts)]
pub struct DelegateAuctioneer<'info> {
    // Auction House instance PDA account.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        bump=auction_house.bump,
        has_one=authority
    )]
    pub auction_house: Account<'info, AuctionHouse>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: The auction house authority can set this to whatever external address they wish.
    /// The auctioneer authority - the program PDA running this auction.
    pub auctioneer_authority: UncheckedAccount<'info>,

    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(
        init,
        payer = authority,
        space = AUCTIONEER_SIZE,
        seeds = [
            AUCTIONEER.as_bytes(),
            auction_house.key().as_ref(),
            auctioneer_authority.key().as_ref()
        ],
        bump
    )]
    pub ah_auctioneer_pda: Account<'info, Auctioneer>,

    pub system_program: Program<'info, System>,
}

pub fn delegate_auctioneer<'info>(
    ctx: Context<'_, '_, '_, 'info, DelegateAuctioneer<'info>>,
    scopes: Vec<AuthorityScope>,
) -> Result<()> {
    if scopes.len() > MAX_NUM_SCOPES {
        return Err(AuctionHouseError::TooManyScopes.into());
    }

    let auction_house = &mut ctx.accounts.auction_house;
    auction_house.has_auctioneer = true;
    auction_house.auctioneer_pda_bump = *ctx
        .bumps
        .get("ah_auctioneer_pda")
        .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?;

    let auctioneer = &mut ctx.accounts.ah_auctioneer_pda;
    auctioneer.auctioneer_authority = ctx.accounts.auctioneer_authority.key();
    auctioneer.auction_house = ctx.accounts.auction_house.key();

    // Set all scopes false and then update as true the ones passed into the handler.
    auctioneer.scopes = [false; MAX_NUM_SCOPES];
    for scope in scopes {
        auctioneer.scopes[scope as usize] = true;
    }

    Ok(())
}
