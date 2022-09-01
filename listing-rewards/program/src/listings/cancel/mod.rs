use crate::{
    constants::{LISTING, REWARD_CENTER},
    state::{Listing, RewardCenter},
    MetadataAccount,
};
use anchor_lang::{prelude::*, InstructionData};
use anchor_spl::token::{Mint, Token, TokenAccount};
use mpl_auction_house::{
    constants::{AUCTIONEER, FEE_PAYER, PREFIX},
    cpi::accounts::AuctioneerCancel,
    instruction::AuctioneerCancel as AuctioneerCancelParams,
    program::AuctionHouse as AuctionHouseProgram,
    AuctionHouse,
};
use solana_program::{instruction::Instruction, program::invoke_signed};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CancelListingParams {
    pub price: u64,
    pub token_size: u64,
}

#[derive(Accounts, Clone)]
#[instruction(cancel_listing_params: CancelListingParams)]
pub struct CancelListing<'info> {
    /// User wallet account.
    #[account(mut)]
    pub wallet: Signer<'info>,

    /// The Listing Config used for listing settings
    #[account(
        seeds = [
            LISTING.as_bytes(),
            wallet.key().as_ref(),
            metadata.key().as_ref(),
            reward_center.key().as_ref(),
        ],
        bump = listing.bump,
    )]
    pub listing: Account<'info, Listing>,

    /// Metaplex metadata account decorating SPL mint account.
    pub metadata: Box<Account<'info, MetadataAccount>>,

    /// SPL token account containing the token of the sale to be canceled.
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// Token mint account of SPL token.
    pub token_mint: Box<Account<'info, Mint>>,

    /// CHECK: Validated as a signer in auction_house program cancel_logic.
    /// Auction House instance authority account.
    pub authority: UncheckedAccount<'info>,

    /// The auctioneer program PDA running this auction.
    #[account(
        seeds = [
            REWARD_CENTER.as_bytes(), 
            auction_house.key().as_ref()
        ], 
        bump = reward_center.bump
    )]
    pub reward_center: Box<Account<'info, RewardCenter>>,

    /// Auction House instance PDA account.
    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        seeds::program = auction_house_program,
        bump = auction_house.bump,
        has_one = authority,
        has_one = auction_house_fee_account
    )]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: Validated in cancel_logic.
    /// Auction House instance fee account.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            FEE_PAYER.as_bytes()
        ],
        seeds::program = auction_house_program,
        bump = auction_house.fee_payer_bump
    )]
    pub auction_house_fee_account: UncheckedAccount<'info>,

    /// CHECK: Validated in cancel_logic.
    /// Trade state PDA account representing the bid or ask to be canceled.
    #[account(mut)]
    pub trade_state: UncheckedAccount<'info>,

    /// CHECK: Validated in cancel_logic.
    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(
        seeds = [
            AUCTIONEER.as_bytes(),
            auction_house.key().as_ref(),
            reward_center.key().as_ref()
        ],
        seeds::program = auction_house_program,
        bump = auction_house.auctioneer_pda_bump
    )]
    pub ah_auctioneer_pda: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub auction_house_program: Program<'info, AuctionHouseProgram>,
}

pub fn handler(
    ctx: Context<CancelListing>,
    CancelListingParams { price, token_size }: CancelListingParams,
) -> Result<()> {
    let reward_center = &ctx.accounts.reward_center;
    let auction_house = &ctx.accounts.auction_house;
    let clock = Clock::get()?;
    let auction_house_key = auction_house.key();

    let listing = &mut ctx.accounts.listing;

    listing.canceled_at = Some(clock.unix_timestamp);

    let reward_center_signer_seeds: &[&[&[u8]]] = &[&[
        REWARD_CENTER.as_bytes(),
        auction_house_key.as_ref(),
        &[reward_center.bump],
    ]];

    let auction_house_program = ctx.accounts.auction_house_program.to_account_info();

    let cancel_listing_ctx_accounts = AuctioneerCancel {
        wallet: ctx.accounts.wallet.to_account_info(),
        token_account: ctx.accounts.token_account.to_account_info(),
        token_mint: ctx.accounts.token_mint.to_account_info(),
        auction_house: ctx.accounts.auction_house.to_account_info(),
        auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
        trade_state: ctx.accounts.trade_state.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        auctioneer_authority: ctx.accounts.reward_center.to_account_info(),
        ah_auctioneer_pda: ctx.accounts.ah_auctioneer_pda.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };

    let cancel_listing_params = AuctioneerCancelParams {
        buyer_price: price,
        token_size,
    };

    let cancel_listing_ix = Instruction {
        program_id: auction_house_program.key(),
        data: cancel_listing_params.data(),
        accounts: cancel_listing_ctx_accounts
            .to_account_metas(None)
            .into_iter()
            .zip(cancel_listing_ctx_accounts.to_account_infos())
            .map(|mut pair| {
                pair.0.is_signer = pair.1.is_signer;
                if pair.0.pubkey == ctx.accounts.reward_center.key() {
                    pair.0.is_signer = true;
                }
                pair.0
            })
            .collect(),
    };

    invoke_signed(
        &cancel_listing_ix,
        &cancel_listing_ctx_accounts.to_account_infos(),
        &reward_center_signer_seeds,
    )?;

    Ok(())
}
