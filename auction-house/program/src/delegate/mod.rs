use anchor_lang::{prelude::*, AnchorDeserialize};

use crate::{constants::*, errors::*, AuctionHouse, Auctioneer, AuthorityScope};

/// Accounts for the [`delegate_auctioneer` handler](auction_house/fn.delegate_auctioneer.html).
#[derive(Accounts)]
#[instruction(ah_auctioneer_pda_bump: u8)]
pub struct DelegateAuctioneer<'info> {
    // Auction House instance PDA account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], bump=auction_house.bump, has_one=authority)]
    pub auction_house: Account<'info, AuctionHouse>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// The auctioneer program PDA running this auction.
    pub auctioneer_authority: UncheckedAccount<'info>,

    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(init, payer = authority, space = AUCTIONEER_SIZE,  seeds = [AUCTIONEER.as_bytes(), auction_house.key().as_ref(), auctioneer_authority.key().as_ref()], bump = ah_auctioneer_pda_bump)]
    pub ah_auctioneer_pda: Account<'info, Auctioneer>,

    pub system_program: Program<'info, System>,
}

pub fn delegate_auctioneer<'info>(
    ctx: Context<'_, '_, '_, 'info, DelegateAuctioneer<'info>>,
    _ah_auctioneer_pda_bump: u8,
    scopes: Box<Vec<AuthorityScope>>,
) -> ProgramResult {
    if scopes.len() > MAX_NUM_SCOPES {
        return Err(ErrorCode::TooManyScopes.into());
    }

    let auction_house = &mut ctx.accounts.auction_house;
    auction_house.has_auctioneer = true;

    let auctioneer = &mut ctx.accounts.ah_auctioneer_pda;
    auctioneer.authority = ctx.accounts.auctioneer_authority.key();
    auctioneer.auction_house = ctx.accounts.auction_house.key();
    auctioneer.scopes = *scopes;

    Ok(())
}
