use anchor_lang::{prelude::*, AnchorDeserialize};

use crate::{constants::*, errors::*, AuctionHouse, Auctioneer, AuthorityScope};

#[derive(Accounts)]
#[instruction(ah_auctioneer_pda_bump: u8)]
pub struct UpdateAuctioneer<'info> {
    // Auction House instance PDA account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], bump=auction_house.bump, has_one=authority)]
    pub auction_house: Account<'info, AuctionHouse>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// The auctioneer program PDA running this auction.
    pub auctioneer_authority: UncheckedAccount<'info>,

    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(mut, seeds = [AUCTIONEER.as_bytes(), auction_house.key().as_ref(), auctioneer_authority.key().as_ref()], bump = ah_auctioneer_pda_bump, has_one=auctioneer_authority)]
    pub ah_auctioneer_pda: Account<'info, Auctioneer>,

    pub system_program: Program<'info, System>,
}

pub fn update_auctioneer<'info>(
    ctx: Context<'_, '_, '_, 'info, UpdateAuctioneer<'info>>,
    _ah_auctioneer_pda_bump: u8,
    scopes: Box<Vec<AuthorityScope>>,
) -> ProgramResult {
    if scopes.len() > MAX_NUM_SCOPES {
        return Err(ErrorCode::TooManyScopes.into());
    }

    let auction_house = &mut ctx.accounts.auction_house;
    if !auction_house.has_auctioneer {
        return Err(ErrorCode::AuctionHouseNotDelegated.into());
    }

    let auctioneer = &mut ctx.accounts.ah_auctioneer_pda;
    auctioneer.auctioneer_authority = ctx.accounts.auctioneer_authority.key();
    auctioneer.auction_house = ctx.accounts.auction_house.key();

    // Set all scopes false and then update as true the ones passed into the handler.
    auctioneer.scopes = [false; 7];
    for scope in *scopes {
        auctioneer.scopes[scope as usize] = true;
    }

    Ok(())
}
