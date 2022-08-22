use crate::MetadataAccount;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use mpl_auction_house::{
    constants::{AUCTIONEER, FEE_PAYER, PREFIX, SIGNER, TREASURY},
    program::AuctionHouse as AuctionHouseProgram,
    AuctionHouse, Auctioneer,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ExecuteSaleParams {
    pub escrow_payment_bump: u8,
    pub free_trade_state_bump: u8,
    pub program_as_signer_bump: u8,
    pub reward_center_bump: u8,
    pub price: u64,
    pub token_size: u64,
}

#[derive(Accounts, Clone)]
#[instruction(execute_sale_params: ExecuteSaleParams)]
pub struct ExecuteSale<'info> {
    // Accounts passed into Auction House CPI call
    /// CHECK: Verified through CPI
    /// Buyer user wallet account.
    #[account(mut)]
    pub buyer: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Seller user wallet account.
    #[account(mut)]
    pub seller: UncheckedAccount<'info>,

    ///Token account where the SPL token is stored.
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// Token mint account for the SPL token.
    pub token_mint: Box<Account<'info, Mint>>,

    /// Metaplex metadata account decorating SPL mint account.
    pub metadata: Box<Account<'info, MetadataAccount>>,

    /// Auction House treasury mint account.
    pub treasury_mint: Box<Account<'info, Mint>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Buyer escrow payment account.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            buyer.key().as_ref()
        ], 
        seeds::program = auction_house_program,
        bump = execute_sale_params.escrow_payment_bump
    )]
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
    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ], 
        seeds::program = auction_house_program,
        bump = auction_house.bump,
        has_one = treasury_mint,
        has_one = auction_house_treasury,
        has_one = auction_house_fee_account
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
        seeds::program = auction_house_program, 
        bump = auction_house.fee_payer_bump
    )]
    pub auction_house_fee_account: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Auction House instance treasury account.
    #[account(
        mut, 
        seeds = [
            PREFIX.as_bytes(), 
            auction_house.key().as_ref(), 
            TREASURY.as_bytes()
        ], 
        seeds::program = auction_house_program, 
        bump=auction_house.treasury_bump
    )]
    pub auction_house_treasury: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Buyer trade state PDA account encoding the buy order.
    #[account(mut)]
    pub buyer_trade_state: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Seller trade state PDA account encoding the sell order.
    #[account(
        mut, 
        seeds = [
            PREFIX.as_bytes(), 
            seller.key().as_ref(), 
            auction_house.key().as_ref(), 
            token_account.key().as_ref(), 
            auction_house.treasury_mint.as_ref(), 
            token_mint.key().as_ref(), 
            &u64::MAX.to_le_bytes(), 
            &execute_sale_params.token_size.to_le_bytes()
        ],
        seeds::program = auction_house_program, 
        bump = seller_trade_state.to_account_info().data.borrow()[0]
    )]
    pub seller_trade_state: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Free seller trade state PDA account encoding a free sell order.
    #[account(
        mut, 
        seeds = [
            PREFIX.as_bytes(), 
            seller.key().as_ref(), 
            auction_house.key().as_ref(), 
            token_account.key().as_ref(), 
            auction_house.treasury_mint.as_ref(), 
            token_mint.key().as_ref(), 
            &0u64.to_le_bytes(), 
            &execute_sale_params.token_size.to_le_bytes()
        ], 
        seeds::program = auction_house_program, 
        bump = execute_sale_params.free_trade_state_bump
    )]
    pub free_trade_state: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// The auctioneer program PDA running this auction.
    #[account(
        seeds = [
            AUCTIONEER.as_bytes(), 
            auction_house.key().as_ref()
        ], 
        bump = execute_sale_params.reward_center_bump
    )]
    pub reward_center: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
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
    pub ah_auctioneer_pda: Account<'info, Auctioneer>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(
        seeds = [
            PREFIX.as_bytes(), 
            SIGNER.as_bytes()
        ], 
        seeds::program = auction_house_program, 
        bump = execute_sale_params.program_as_signer_bump
    )]
    pub program_as_signer: UncheckedAccount<'info>,

    /// Auction House Program
    pub auction_house_program: Program<'info, AuctionHouseProgram>,
    /// Token Program
    pub token_program: Program<'info, Token>,
    /// System Program
    pub system_program: Program<'info, System>,
    /// Associated Token Program
    pub ata_program: Program<'info, AssociatedToken>,
    /// Rent
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<ExecuteSale>,
    ExecuteSaleParams {
        price,
        escrow_payment_bump,
        free_trade_state_bump,
        program_as_signer_bump,
        reward_center_bump,
        token_size,
    }: ExecuteSaleParams,
) -> Result<()> {

    Ok(())
}
