use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use mpl_auction_house::{
    self,
    constants::{
        PREFIX,
        FEE_PAYER,
        TREASURY,
        SIGNER,
    },
    //auction_house::{
    cpi::{
        accounts::{
            Withdraw as AHWithdraw,
            Deposit as AHDeposit,
            Cancel as AHCancel,
            ExecuteSale as AHExecuteSale,
            Sell as AHSell,
            Buy as AHBuy,
            PublicBuy as AHPublicBuy,
        },
    },
    program::AuctionHouse as AuctionHouseProgram,//program::auction_house as AuctionHouseProgram,
        //program::auction_house,
    //},
    AuctionHouse,
};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

//pub const PREFIX: &str = "auctioneer";

#[program]
pub mod auctioneer {
    use super::*;

    /// Withdraw `amount` from the escrow payment account for your specific wallet.
    pub fn withdraw<'info>(
        ctx: Context<'_, '_, '_, 'info, Withdraw<'info>>,
        escrow_payment_bump: u8,
        amount: u64,
    ) -> ProgramResult {
        let cpi_program = ctx.accounts.auction_house_program.to_account_info();
        let cpi_accounts = AHWithdraw {
            wallet: ctx.accounts.wallet.to_account_info(),
            receipt_account: ctx.accounts.receipt_account.to_account_info(),
            escrow_payment_account: ctx.accounts.escrow_payment_account.to_account_info(),
            treasury_mint: ctx.accounts.treasury_mint.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            auction_house: ctx.accounts.auction_house.to_account_info(),
            auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            ata_program: ctx.accounts.ata_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        mpl_auction_house::cpi::withdraw(cpi_ctx, escrow_payment_bump, amount)
    }

    /// Deposit `amount` into the escrow payment account for your specific wallet.
    pub fn deposit<'info>(
        ctx: Context<'_, '_, '_, 'info, Deposit<'info>>,
        escrow_payment_bump: u8,
        amount: u64,
    ) -> ProgramResult {
        let cpi_program = ctx.accounts.auction_house_program.to_account_info();
        let cpi_accounts = AHDeposit {
            wallet: ctx.accounts.wallet.to_account_info(),
            payment_account: ctx.accounts.payment_account.to_account_info(),
            transfer_authority: ctx.accounts.transfer_authority.to_account_info(),
            escrow_payment_account: ctx.accounts.escrow_payment_account.to_account_info(),
            treasury_mint: ctx.accounts.treasury_mint.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            auction_house: ctx.accounts.auction_house.to_account_info(),
            auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        mpl_auction_house::cpi::deposit(cpi_ctx, escrow_payment_bump, amount)
    }

    /// Cancel a bid or ask by revoking the token delegate, transferring all lamports from the trade state account to the fee payer, and setting the trade state account data to zero so it can be garbage collected.
    pub fn cancel<'info>(
        ctx: Context<'_, '_, '_, 'info, Cancel<'info>>,
        buyer_price: u64,
        token_size: u64,
    ) -> ProgramResult {
        let cpi_program = ctx.accounts.auction_house_program.to_account_info();
        let cpi_accounts = AHCancel {
            wallet: ctx.accounts.wallet.to_account_info(),
            token_account: ctx.accounts.token_account.to_account_info(),
            token_mint: ctx.accounts.token_mint.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            auction_house: ctx.accounts.auction_house.to_account_info(),
            auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
            trade_state: ctx.accounts.trade_state.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        mpl_auction_house::cpi::cancel(cpi_ctx, buyer_price, token_size)
    }

    /// Execute sale between provided buyer and seller trade state accounts transferring funds to seller wallet and token to buyer wallet.
    #[inline(never)]
    pub fn execute_sale<'info>(
        ctx: Context<'_, '_, '_, 'info, ExecuteSale<'info>>,
        escrow_payment_bump: u8,
        free_trade_state_bump: u8,
        program_as_signer_bump: u8,
        buyer_price: u64,
        token_size: u64,
    ) -> ProgramResult {
        let cpi_program = ctx.accounts.auction_house_program.to_account_info();
        let cpi_accounts = AHExecuteSale {
            buyer: ctx.accounts.buyer.to_account_info(),
            seller: ctx.accounts.seller.to_account_info(),
            token_account: ctx.accounts.token_account.to_account_info(),
            token_mint: ctx.accounts.token_mint.to_account_info(),
            metadata: ctx.accounts.metadata.to_account_info(),
            treasury_mint: ctx.accounts.treasury_mint.to_account_info(),
            escrow_payment_account: ctx.accounts.escrow_payment_account.to_account_info(),
            seller_payment_receipt_account: ctx.accounts.seller_payment_receipt_account.to_account_info(),
            buyer_receipt_token_account: ctx.accounts.buyer_receipt_token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            auction_house: ctx.accounts.auction_house.to_account_info(),
            auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
            auction_house_treasury: ctx.accounts.auction_house_treasury.to_account_info(),
            buyer_trade_state: ctx.accounts.buyer_trade_state.to_account_info(),
            seller_trade_state: ctx.accounts.seller_trade_state.to_account_info(),
            free_trade_state: ctx.accounts.free_trade_state.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            ata_program: ctx.accounts.ata_program.to_account_info(),
            program_as_signer: ctx.accounts.program_as_signer.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        mpl_auction_house::cpi::execute_sale(cpi_ctx, escrow_payment_bump, free_trade_state_bump, program_as_signer_bump, buyer_price, token_size)
    }

    /// Create a sell bid by creating a `seller_trade_state` account and approving the program as the token delegate.
    pub fn sell<'info>(
        ctx: Context<'_, '_, '_, 'info, Sell<'info>>,
        trade_state_bump: u8,
        free_trade_state_bump: u8,
        program_as_signer_bump: u8,
        buyer_price: u64,
        token_size: u64,
    ) -> ProgramResult {
        let cpi_program = ctx.accounts.auction_house_program.to_account_info();
        let cpi_accounts = AHSell {
            wallet: ctx.accounts.wallet.to_account_info(),
            token_account: ctx.accounts.token_account.to_account_info(),
            metadata: ctx.accounts.metadata.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            auction_house: ctx.accounts.auction_house.to_account_info(),
            auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
            seller_trade_state: ctx.accounts.seller_trade_state.to_account_info(),
            free_seller_trade_state: ctx.accounts.free_seller_trade_state.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            program_as_signer: ctx.accounts.program_as_signer.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        mpl_auction_house::cpi::sell(cpi_ctx, trade_state_bump, free_trade_state_bump, program_as_signer_bump, buyer_price, token_size)
    }

    /// Create a private buy bid by creating a `buyer_trade_state` account and an `escrow_payment` account and funding the escrow with the necessary SOL or SPL token amount.
    pub fn buy<'info>(
        ctx: Context<'_, '_, '_, 'info, Buy<'info>>,
        trade_state_bump: u8,
        escrow_payment_bump: u8,
        buyer_price: u64,
        token_size: u64,
    ) -> ProgramResult {
        let cpi_program = ctx.accounts.auction_house_program.to_account_info();
        let cpi_accounts = AHBuy {
            wallet: ctx.accounts.wallet.to_account_info(),
            payment_account: ctx.accounts.payment_account.to_account_info(),
            transfer_authority: ctx.accounts.transfer_authority.to_account_info(),
            treasury_mint: ctx.accounts.treasury_mint.to_account_info(),
            token_account: ctx.accounts.token_account.to_account_info(),
            metadata: ctx.accounts.metadata.to_account_info(),
            escrow_payment_account: ctx.accounts.escrow_payment_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            auction_house: ctx.accounts.auction_house.to_account_info(),
            auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
            buyer_trade_state: ctx.accounts.buyer_trade_state.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        mpl_auction_house::cpi::buy(cpi_ctx, trade_state_bump, escrow_payment_bump, buyer_price, token_size)
    }

    /// Create a public buy bid by creating a `public_buyer_trade_state` account and an `escrow_payment` account and funding the escrow with the necessary SOL or SPL token amount.
    pub fn public_buy<'info>(
        ctx: Context<'_, '_, '_, 'info, PublicBuy<'info>>,
        trade_state_bump: u8,
        escrow_payment_bump: u8,
        buyer_price: u64,
        token_size: u64,
    ) -> ProgramResult {
        let cpi_program = ctx.accounts.auction_house_program.to_account_info();
        let cpi_accounts = AHPublicBuy {
            wallet: ctx.accounts.wallet.to_account_info(),
            payment_account: ctx.accounts.payment_account.to_account_info(),
            transfer_authority: ctx.accounts.transfer_authority.to_account_info(),
            treasury_mint: ctx.accounts.treasury_mint.to_account_info(),
            token_account: ctx.accounts.token_account.to_account_info(),
            metadata: ctx.accounts.metadata.to_account_info(),
            escrow_payment_account: ctx.accounts.escrow_payment_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            auction_house: ctx.accounts.auction_house.to_account_info(),
            auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
            buyer_trade_state: ctx.accounts.buyer_trade_state.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        mpl_auction_house::cpi::public_buy(cpi_ctx, trade_state_bump, escrow_payment_bump, buyer_price, token_size)
    }
}

/// Accounts for the [`withdraw` handler](auction_house/fn.withdraw.html).
#[derive(Accounts)]
#[instruction(escrow_payment_bump: u8)]
pub struct Withdraw<'info> {
    /// Auction House Program
    pub auction_house_program: Program<'info, AuctionHouseProgram>,
    /// User wallet account.
    pub wallet: UncheckedAccount<'info>,
    /// SPL token account or native SOL account to transfer funds to. If the account is a native SOL account, this is the same as the wallet address.
    #[account(mut)]
    pub receipt_account: UncheckedAccount<'info>,
    /// Buyer escrow payment account PDA.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), wallet.key().as_ref()], bump=escrow_payment_bump)]
    pub escrow_payment_account: UncheckedAccount<'info>,
    /// Auction House instance treasury mint account.
    pub treasury_mint: Account<'info, Mint>,
    /// Auction House instance authority account.
    pub authority: UncheckedAccount<'info>,
    /// Auction House instance PDA account.
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], bump=auction_house.bump, has_one=authority, has_one=treasury_mint, has_one=auction_house_fee_account)]
    pub auction_house: Account<'info, AuctionHouse>,
    /// Auction House instance fee account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], bump=auction_house.fee_payer_bump)]
    pub auction_house_fee_account: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub ata_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

/// Accounts for the [`deposit` handler](auction_house/fn.deposit.html).
#[derive(Accounts)]
#[instruction(escrow_payment_bump: u8)]
pub struct Deposit<'info> {
    /// Auction House Program
    pub auction_house_program: Program<'info, AuctionHouseProgram>,
    /// User wallet account.
    pub wallet: Signer<'info>,
    /// User SOL or SPL account to transfer funds from.
    #[account(mut)]
    pub payment_account: UncheckedAccount<'info>,
    /// SPL token account transfer authority.
    pub transfer_authority: UncheckedAccount<'info>,
    /// Buyer escrow payment account PDA.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), wallet.key().as_ref()], bump=escrow_payment_bump)]
    pub escrow_payment_account: UncheckedAccount<'info>,
    /// Auction House instance treasury mint account.
    pub treasury_mint: Account<'info, Mint>,
    /// Auction House instance authority account.
    pub authority: UncheckedAccount<'info>,
    /// Auction House instance PDA account.
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], bump=auction_house.bump, has_one=authority, has_one=treasury_mint, has_one=auction_house_fee_account)]
    pub auction_house: Account<'info, AuctionHouse>,
    /// Auction House instance fee account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], bump=auction_house.fee_payer_bump)]
    pub auction_house_fee_account: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// Accounts for the [`cancel` handler](auction_house/fn.cancel.html).
#[derive(Accounts)]
#[instruction(buyer_price: u64, token_size: u64)]
pub struct Cancel<'info> {
    /// Auction House Program
    pub auction_house_program: Program<'info, AuctionHouseProgram>,
    /// User wallet account.
    #[account(mut)]
    pub wallet: UncheckedAccount<'info>,
    /// SPL token account containing the token of the sale to be canceled.
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    /// Token mint account of SPL token.
    pub token_mint: Account<'info, Mint>,
    /// Auction House instance authority account.
    pub authority: UncheckedAccount<'info>,
    /// Auction House instance PDA account.
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], bump=auction_house.bump, has_one=authority, has_one=auction_house_fee_account)]
    pub auction_house: Account<'info, AuctionHouse>,
    /// Auction House instance fee account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], bump=auction_house.fee_payer_bump)]
    pub auction_house_fee_account: UncheckedAccount<'info>,
    /// Trade state PDA account representing the bid or ask to be canceled.
    #[account(mut)]
    pub trade_state: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

/// Accounts for the [`execute_sale` handler](auction_house/fn.execute_sale.html).
#[derive(Accounts)]
#[instruction(escrow_payment_bump: u8, free_trade_state_bump: u8, program_as_signer_bump: u8, buyer_price: u64, token_size: u64)]
pub struct ExecuteSale<'info> {
    /// Auction House Program
    pub auction_house_program: Program<'info, AuctionHouseProgram>,
    /// Buyer user wallet account.
    #[account(mut)]
    pub buyer: UncheckedAccount<'info>,
    /// Seller user wallet account.
    #[account(mut)]
    pub seller: UncheckedAccount<'info>,
    // cannot mark these as real Accounts or else we blow stack size limit
    ///Token account where the SPL token is stored.
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    /// Token mint account for the SPL token.
    pub token_mint: UncheckedAccount<'info>,
    /// Metaplex metadata account decorating SPL mint account.
    pub metadata: UncheckedAccount<'info>,
    // cannot mark these as real Accounts or else we blow stack size limit
    /// Auction House treasury mint account.
    pub treasury_mint: UncheckedAccount<'info>,
    /// Buyer escrow payment account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), buyer.key().as_ref()], bump=escrow_payment_bump)]
    pub escrow_payment_account: UncheckedAccount<'info>,
    /// Seller SOL or SPL account to receive payment at.
    #[account(mut)]
    pub seller_payment_receipt_account: UncheckedAccount<'info>,
    /// Buyer SPL token account to receive purchased item at.
    #[account(mut)]
    pub buyer_receipt_token_account: UncheckedAccount<'info>,
    /// Auction House instance authority.
    pub authority: UncheckedAccount<'info>,
    /// Auction House instance PDA account.
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], bump=auction_house.bump, has_one=authority, has_one=treasury_mint, has_one=auction_house_treasury, has_one=auction_house_fee_account)]
    pub auction_house: Box<Account<'info, AuctionHouse>>,
    /// Auction House instance fee account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], bump=auction_house.fee_payer_bump)]
    pub auction_house_fee_account: UncheckedAccount<'info>,
    /// Auction House instance treasury account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), TREASURY.as_bytes()], bump=auction_house.treasury_bump)]
    pub auction_house_treasury: UncheckedAccount<'info>,
    /// Buyer trade state PDA account encoding the buy order.
    #[account(mut)]
    pub buyer_trade_state: UncheckedAccount<'info>,
    /// Seller trade state PDA account encoding the sell order.
    #[account(mut, seeds=[PREFIX.as_bytes(), seller.key().as_ref(), auction_house.key().as_ref(), token_account.key().as_ref(), auction_house.treasury_mint.as_ref(), token_mint.key().as_ref(), &buyer_price.to_le_bytes(), &token_size.to_le_bytes()], bump=seller_trade_state.to_account_info().data.borrow()[0])]
    pub seller_trade_state: UncheckedAccount<'info>,
    /// Free seller trade state PDA account encoding a free sell order.
    #[account(mut, seeds=[PREFIX.as_bytes(), seller.key().as_ref(), auction_house.key().as_ref(), token_account.key().as_ref(), auction_house.treasury_mint.as_ref(), token_mint.key().as_ref(), &0u64.to_le_bytes(), &token_size.to_le_bytes()], bump=free_trade_state_bump)]
    pub free_trade_state: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub ata_program: Program<'info, AssociatedToken>,
    #[account(seeds=[PREFIX.as_bytes(), SIGNER.as_bytes()], bump=program_as_signer_bump)]
    pub program_as_signer: UncheckedAccount<'info>,
    pub rent: Sysvar<'info, Rent>,
}

/// Accounts for the [`sell` handler](auction_house/fn.sell.html).
#[derive(Accounts)]
#[instruction(trade_state_bump: u8, free_trade_state_bump: u8, program_as_signer_bump: u8, buyer_price: u64, token_size: u64)]
pub struct Sell<'info> {
    /// Auction House Program
    pub auction_house_program: Program<'info, AuctionHouseProgram>,
    /// User wallet account.
    pub wallet: UncheckedAccount<'info>,
    #[account(mut)]
    /// SPL token account containing token for sale.
    pub token_account: Account<'info, TokenAccount>,
    /// Metaplex metadata account decorating SPL mint account.
    pub metadata: UncheckedAccount<'info>,
    /// Auction House authority account.
    pub authority: UncheckedAccount<'info>,
    /// Auction House instance PDA account.
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], bump=auction_house.bump, has_one=authority, has_one=auction_house_fee_account)]
    pub auction_house: Account<'info, AuctionHouse>,
    /// Auction House instance fee account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], bump=auction_house.fee_payer_bump)]
    pub auction_house_fee_account: UncheckedAccount<'info>,
    /// Seller trade state PDA account encoding the sell order.
    #[account(mut, seeds=[PREFIX.as_bytes(), wallet.key().as_ref(), auction_house.key().as_ref(), token_account.key().as_ref(), auction_house.treasury_mint.as_ref(), token_account.mint.as_ref(), &buyer_price.to_le_bytes(), &token_size.to_le_bytes()], bump=trade_state_bump)]
    pub seller_trade_state: UncheckedAccount<'info>,
    /// Free seller trade state PDA account encoding a free sell order.
    #[account(mut, seeds=[PREFIX.as_bytes(), wallet.key().as_ref(), auction_house.key().as_ref(), token_account.key().as_ref(), auction_house.treasury_mint.as_ref(), token_account.mint.as_ref(), &0u64.to_le_bytes(), &token_size.to_le_bytes()], bump=free_trade_state_bump)]
    pub free_seller_trade_state: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    #[account(seeds=[PREFIX.as_bytes(), SIGNER.as_bytes()], bump=program_as_signer_bump)]
    pub program_as_signer: UncheckedAccount<'info>,
    pub rent: Sysvar<'info, Rent>,
}

/// Accounts for the [`private_bid` handler](fn.private_bid.html).
#[derive(Accounts)]
#[instruction(trade_state_bump: u8, escrow_payment_bump: u8, buyer_price: u64, token_size: u64)]
pub struct Buy<'info> {
    /// Auction House Program
    pub auction_house_program: Program<'info, AuctionHouseProgram>,
    wallet: Signer<'info>,
    #[account(mut)]
    payment_account: UncheckedAccount<'info>,
    transfer_authority: UncheckedAccount<'info>,
    treasury_mint: Account<'info, Mint>,
    token_account: Account<'info, TokenAccount>,
    metadata: UncheckedAccount<'info>,
    #[account(mut, seeds = [PREFIX.as_bytes(), auction_house.key().as_ref(), wallet.key().as_ref()], bump = escrow_payment_bump)]
    escrow_payment_account: UncheckedAccount<'info>,
    authority: UncheckedAccount<'info>,
    #[account(seeds = [PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], bump = auction_house.bump, has_one = authority, has_one = treasury_mint, has_one = auction_house_fee_account)]
    auction_house: Account<'info, AuctionHouse>,
    #[account(mut, seeds = [PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], bump = auction_house.fee_payer_bump)]
    auction_house_fee_account: UncheckedAccount<'info>,
    #[account(mut, seeds = [PREFIX.as_bytes(), wallet.key().as_ref(), auction_house.key().as_ref(), token_account.key().as_ref(), treasury_mint.key().as_ref(), token_account.mint.as_ref(), buyer_price.to_le_bytes().as_ref(), token_size.to_le_bytes().as_ref()], bump = trade_state_bump)]
    buyer_trade_state: UncheckedAccount<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

/// Accounts for the [`public_bid` handler](fn.public_bid.html).
#[derive(Accounts)]
#[instruction(trade_state_bump: u8, escrow_payment_bump: u8, buyer_price: u64, token_size: u64)]
pub struct PublicBuy<'info> {
    /// Auction House Program
    pub auction_house_program: Program<'info, AuctionHouseProgram>,
    wallet: Signer<'info>,
    #[account(mut)]
    payment_account: UncheckedAccount<'info>,
    transfer_authority: UncheckedAccount<'info>,
    treasury_mint: Account<'info, Mint>,
    token_account: Account<'info, TokenAccount>,
    metadata: UncheckedAccount<'info>,
    #[account(mut, seeds = [PREFIX.as_bytes(), auction_house.key().as_ref(), wallet.key().as_ref()], bump = escrow_payment_bump)]
    escrow_payment_account: UncheckedAccount<'info>,
    authority: UncheckedAccount<'info>,
    #[account(seeds = [PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], bump = auction_house.bump, has_one = authority, has_one = treasury_mint, has_one = auction_house_fee_account)]
    auction_house: Account<'info, AuctionHouse>,
    #[account(mut, seeds = [PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], bump = auction_house.fee_payer_bump)]
    auction_house_fee_account: UncheckedAccount<'info>,
    #[account(mut, seeds = [PREFIX.as_bytes(), wallet.key().as_ref(), auction_house.key().as_ref(), treasury_mint.key().as_ref(), token_account.mint.as_ref(), buyer_price.to_le_bytes().as_ref(), token_size.to_le_bytes().as_ref()], bump = trade_state_bump)]
    buyer_trade_state: UncheckedAccount<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}
