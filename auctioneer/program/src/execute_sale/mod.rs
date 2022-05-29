use anchor_lang::{prelude::*, AnchorDeserialize, InstructionData};
use anchor_spl::{associated_token::AssociatedToken, token::Token};

use mpl_auction_house::{
    self,
    constants::{AUCTIONEER, FEE_PAYER, PREFIX, SIGNER, TREASURY},
    cpi::accounts::AuctioneerExecuteSale as AHExecuteSale,
    program::AuctionHouse as AuctionHouseProgram,
    AuctionHouse,
};

use crate::{constants::*, sell::config::*, utils::*};

use solana_program::program::invoke_signed;

#[derive(Accounts)]
#[instruction(escrow_payment_bump: u8, free_trade_state_bump: u8, program_as_signer_bump: u8, auctioneer_authority_bump: u8, buyer_price: u64, token_size: u64)]
pub struct AuctioneerExecuteSale<'info> {
    /// Auction House Program
    pub auction_house_program: Program<'info, AuctionHouseProgram>,

    // Accounts used for Auctioneer
    /// The Listing Config used for listing settings
    #[account(
        seeds=[
            LISTING_CONFIG.as_bytes(),
            seller.key().as_ref(),
            auction_house.key().as_ref(),
            token_account.key().as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_mint.key().as_ref(),
            &token_size.to_le_bytes()
        ],
        bump=listing_config.bump,
    )]
    pub listing_config: Box<Account<'info, ListingConfig>>,

    // Accounts passed into Auction House CPI call
    /// CHECK: Verified through CPI
    /// Buyer user wallet account.
    #[account(mut)]
    pub buyer: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Seller user wallet account.
    #[account(mut)]
    pub seller: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    // cannot mark these as real Accounts or else we blow stack size limit
    ///Token account where the SPL token is stored.
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Token mint account for the SPL token.
    pub token_mint: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Metaplex metadata account decorating SPL mint account.
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    // cannot mark these as real Accounts or else we blow stack size limit
    /// Auction House treasury mint account.
    pub treasury_mint: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Buyer escrow payment account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), buyer.key().as_ref()], seeds::program=auction_house_program, bump=escrow_payment_bump)]
    pub escrow_payment_account: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Seller SOL or SPL account to receive payment at.
    #[account(mut)]
    pub seller_payment_receipt_account: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Buyer SPL token account to receive purchased item at.
    #[account(mut)]
    pub buyer_receipt_token_account: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Auction House instance authority.
    pub authority: UncheckedAccount<'info>,

    /// Auction House instance PDA account.
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], seeds::program=auction_house_program, bump=auction_house.bump, has_one=treasury_mint, has_one=auction_house_treasury, has_one=auction_house_fee_account)]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Auction House instance fee account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], seeds::program=auction_house_program, bump=auction_house.fee_payer_bump)]
    pub auction_house_fee_account: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Auction House instance treasury account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), TREASURY.as_bytes()], seeds::program=auction_house_program, bump=auction_house.treasury_bump)]
    pub auction_house_treasury: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Buyer trade state PDA account encoding the buy order.
    #[account(mut)]
    pub buyer_trade_state: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Seller trade state PDA account encoding the sell order.
    #[account(mut, seeds=[PREFIX.as_bytes(), seller.key().as_ref(), auction_house.key().as_ref(), token_account.key().as_ref(), auction_house.treasury_mint.as_ref(), token_mint.key().as_ref(), &u64::MAX.to_le_bytes(), &token_size.to_le_bytes()], seeds::program=auction_house_program, bump=seller_trade_state.to_account_info().data.borrow()[0])]
    pub seller_trade_state: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Free seller trade state PDA account encoding a free sell order.
    #[account(mut, seeds=[PREFIX.as_bytes(), seller.key().as_ref(), auction_house.key().as_ref(), token_account.key().as_ref(), auction_house.treasury_mint.as_ref(), token_mint.key().as_ref(), &0u64.to_le_bytes(), &token_size.to_le_bytes()], seeds::program=auction_house_program, bump=free_trade_state_bump)]
    pub free_trade_state: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// The auctioneer program PDA running this auction.
    #[account(seeds = [AUCTIONEER.as_bytes(), auction_house.key().as_ref()], bump=auctioneer_authority_bump)]
    pub auctioneer_authority: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(seeds = [AUCTIONEER.as_bytes(), auction_house.key().as_ref(), auctioneer_authority.key().as_ref()], seeds::program=auction_house_program, bump = auction_house.auctioneer_pda_bump)]
    pub ah_auctioneer_pda: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub ata_program: Program<'info, AssociatedToken>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(seeds=[PREFIX.as_bytes(), SIGNER.as_bytes()], seeds::program=auction_house_program, bump=program_as_signer_bump)]
    pub program_as_signer: UncheckedAccount<'info>,

    pub rent: Sysvar<'info, Rent>,
}

pub fn auctioneer_execute_sale<'info>(
    ctx: Context<'_, '_, '_, 'info, AuctioneerExecuteSale<'info>>,
    escrow_payment_bump: u8,
    free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    auctioneer_authority_bump: u8,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    assert_auction_over(&ctx.accounts.listing_config)?;
    assert_highest_bidder(
        &ctx.accounts.listing_config,
        ctx.accounts.buyer_trade_state.key(),
    )?;

    let cpi_program = ctx.accounts.auction_house_program.to_account_info();
    let cpi_accounts = AHExecuteSale {
        buyer: ctx.accounts.buyer.to_account_info(),
        seller: ctx.accounts.seller.to_account_info(),
        token_account: ctx.accounts.token_account.to_account_info(),
        token_mint: ctx.accounts.token_mint.to_account_info(),
        metadata: ctx.accounts.metadata.to_account_info(),
        treasury_mint: ctx.accounts.treasury_mint.to_account_info(),
        escrow_payment_account: ctx.accounts.escrow_payment_account.to_account_info(),
        seller_payment_receipt_account: ctx
            .accounts
            .seller_payment_receipt_account
            .to_account_info(),
        buyer_receipt_token_account: ctx.accounts.buyer_receipt_token_account.to_account_info(),
        auction_house: ctx.accounts.auction_house.to_account_info(),
        auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
        auction_house_treasury: ctx.accounts.auction_house_treasury.to_account_info(),
        buyer_trade_state: ctx.accounts.buyer_trade_state.to_account_info(),
        seller_trade_state: ctx.accounts.seller_trade_state.to_account_info(),
        free_trade_state: ctx.accounts.free_trade_state.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        auctioneer_authority: ctx.accounts.auctioneer_authority.to_account_info(),
        ah_auctioneer_pda: ctx.accounts.ah_auctioneer_pda.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        ata_program: ctx.accounts.ata_program.to_account_info(),
        program_as_signer: ctx.accounts.program_as_signer.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };

    let execute_sale_data = mpl_auction_house::instruction::AuctioneerExecuteSale {
        escrow_payment_bump,
        _free_trade_state_bump: free_trade_state_bump,
        program_as_signer_bump,
        buyer_price,
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
        data: execute_sale_data.data(),
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
