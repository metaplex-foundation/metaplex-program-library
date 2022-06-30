pub mod config;

use crate::{constants::*, errors::*, sell::config::*};

use anchor_lang::{prelude::*, AnchorDeserialize, InstructionData};
use anchor_spl::token::{Token, TokenAccount};

use mpl_auction_house::{
    self,
    constants::{AUCTIONEER, FEE_PAYER, PREFIX, SIGNER},
    cpi::accounts::AuctioneerSell as AHSell,
    program::AuctionHouse as AuctionHouseProgram,
    AuctionHouse,
};

use solana_program::{clock::UnixTimestamp, program::invoke_signed};

/// Accounts for the [`sell_with_auctioneer` handler](auction_house/fn.sell_with_auctioneer.html).
#[derive(Accounts, Clone)]
#[instruction(trade_state_bump: u8, free_trade_state_bump: u8, program_as_signer_bump: u8, auctioneer_authority_bump: u8, token_size: u64)]
pub struct AuctioneerSell<'info> {
    /// Auction House Program used for CPI call
    pub auction_house_program: Program<'info, AuctionHouseProgram>,

    // Accounts used for Auctioneer
    /// The Listing Config used for listing settings
    #[account(
        init,
        payer=wallet,
        space=LISTING_CONFIG_SIZE,
        seeds=[
            LISTING_CONFIG.as_bytes(),
            wallet.key().as_ref(),
            auction_house.key().as_ref(),
            token_account.key().as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_account.mint.as_ref(),
            &token_size.to_le_bytes()
        ],
        bump,
    )]
    pub listing_config: Account<'info, ListingConfig>,

    // Accounts passed into Auction House CPI call
    /// CHECK: Verified through CPI
    /// User wallet account.
    #[account(mut)]
    pub wallet: UncheckedAccount<'info>,

    /// SPL token account containing token for sale.
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: Verified through CPI
    /// Metaplex metadata account decorating SPL mint account.
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Auction House authority account.
    pub authority: UncheckedAccount<'info>,

    /// Auction House instance PDA account.
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], seeds::program=auction_house_program, bump=auction_house.bump, has_one=auction_house_fee_account)]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Auction House instance fee account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], seeds::program=auction_house_program, bump=auction_house.fee_payer_bump)]
    pub auction_house_fee_account: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Seller trade state PDA account encoding the sell order.
    #[account(mut, seeds=[PREFIX.as_bytes(), wallet.key().as_ref(), auction_house.key().as_ref(), token_account.key().as_ref(), auction_house.treasury_mint.as_ref(), token_account.mint.as_ref(), &u64::MAX.to_le_bytes(), &token_size.to_le_bytes()], seeds::program=auction_house_program, bump=trade_state_bump)]
    pub seller_trade_state: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Free seller trade state PDA account encoding a free sell order.
    #[account(mut, seeds=[PREFIX.as_bytes(), wallet.key().as_ref(), auction_house.key().as_ref(), token_account.key().as_ref(), auction_house.treasury_mint.as_ref(), token_account.mint.as_ref(), &0u64.to_le_bytes(), &token_size.to_le_bytes()], seeds::program=auction_house_program, bump=free_trade_state_bump)]
    pub free_seller_trade_state: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// The auctioneer program PDA running this auction.
    pub auctioneer_authority: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(seeds = [AUCTIONEER.as_bytes(), auction_house.key().as_ref(), auctioneer_authority.key().as_ref()], seeds::program=auction_house_program, bump = auction_house.auctioneer_pda_bump)]
    pub ah_auctioneer_pda: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(seeds=[PREFIX.as_bytes(), SIGNER.as_bytes()], seeds::program=auction_house_program, bump=program_as_signer_bump)]
    pub program_as_signer: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// Create a sell bid by creating a `seller_trade_state` account and approving the program as the token delegate.
pub fn auctioneer_sell<'info>(
    ctx: Context<'_, '_, '_, 'info, AuctioneerSell<'info>>,
    trade_state_bump: u8,
    free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    auctioneer_authority_bump: u8,
    token_size: u64,
    start_time: UnixTimestamp,
    end_time: UnixTimestamp,
    reserve_price: Option<u64>,
    min_bid_increment: Option<u64>,
    time_ext_period: Option<u32>,
    time_ext_delta: Option<u32>,
    allow_high_bid_cancel: Option<bool>,
) -> Result<()> {
    ctx.accounts.listing_config.version = ListingConfigVersion::V0;
    ctx.accounts.listing_config.highest_bid.version = ListingConfigVersion::V0;
    ctx.accounts.listing_config.start_time = start_time;
    ctx.accounts.listing_config.end_time = end_time;
    ctx.accounts.listing_config.reserve_price = reserve_price.unwrap_or(0);
    ctx.accounts.listing_config.min_bid_increment = min_bid_increment.unwrap_or(0);
    ctx.accounts.listing_config.time_ext_period = time_ext_period.unwrap_or(0);
    ctx.accounts.listing_config.time_ext_delta = time_ext_delta.unwrap_or(0);
    ctx.accounts.listing_config.allow_high_bid_cancel = allow_high_bid_cancel.unwrap_or(false);
    ctx.accounts.listing_config.bump = *ctx
        .bumps
        .get("listing_config")
        .ok_or(AuctioneerError::BumpSeedNotInHashMap)?;

    let cpi_program = ctx.accounts.auction_house_program.to_account_info();
    let cpi_accounts = AHSell {
        wallet: ctx.accounts.wallet.to_account_info(),
        token_account: ctx.accounts.token_account.to_account_info(),
        metadata: ctx.accounts.metadata.to_account_info(),
        auction_house: ctx.accounts.auction_house.to_account_info(),
        auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
        seller_trade_state: ctx.accounts.seller_trade_state.to_account_info(),
        free_seller_trade_state: ctx.accounts.free_seller_trade_state.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        auctioneer_authority: ctx.accounts.auctioneer_authority.to_account_info(),
        ah_auctioneer_pda: ctx.accounts.ah_auctioneer_pda.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        program_as_signer: ctx.accounts.program_as_signer.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };

    let sell_data = mpl_auction_house::instruction::AuctioneerSell {
        trade_state_bump,
        free_trade_state_bump,
        program_as_signer_bump,
        token_size,
    };

    let ix = solana_program::instruction::Instruction {
        program_id: cpi_program.key(),
        accounts: cpi_accounts
            .to_account_metas(None)
            .into_iter()
            .zip(cpi_accounts.to_account_infos())
            .map(|mut pair| {
                pair.0.is_signer = pair.1.is_signer;
                if pair.0.pubkey == ctx.accounts.auctioneer_authority.key() {
                    pair.0.is_signer = true;
                }
                pair.0
            })
            .collect(),
        data: sell_data.data(),
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

    invoke_signed(&ix, &cpi_accounts.to_account_infos(), &[&auctioneer_seeds])?;

    Ok(())
}
