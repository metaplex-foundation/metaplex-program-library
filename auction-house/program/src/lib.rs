//! # Metaplex Program Library: Auction House
//! AuctionHouse is a protocol for marketplaces to implement a decentralized sales contract. It is simple, fast and very cheap. AuctionHouse is a Solana program available on Mainnet Beta and Devnet. Anyone can create an AuctionHouse and accept any SPL token they wish.
//!
//! Full docs can be found [here](https://docs.metaplex.com/auction-house/definition).
pub mod bid;
pub mod cancel;
pub mod constants;
pub mod delegate;
pub mod deposit;
pub mod errors;
pub mod execute_sale;
pub mod pda;
pub mod receipt;
pub mod sell;
pub mod state;
pub mod utils;
pub mod withdraw;

pub use state::*;

use crate::{
    bid::*, cancel::*, constants::*, delegate::*, deposit::*, errors::*, execute_sale::*,
    receipt::*, sell::*, utils::*, withdraw::*,
};
use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke_signed, system_instruction},
    AnchorDeserialize, AnchorSerialize,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use spl_token::instruction::revoke;

anchor_lang::declare_id!("hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk");

#[program]
pub mod auction_house {

    use super::*;

    /// Withdraw `amount` from the Auction House Fee Account to a provided destination account.
    pub fn withdraw_from_fee<'info>(
        ctx: Context<'_, '_, '_, 'info, WithdrawFromFee<'info>>,
        amount: u64,
    ) -> ProgramResult {
        let auction_house_fee_account = &ctx.accounts.auction_house_fee_account;
        let fee_withdrawal_destination = &ctx.accounts.fee_withdrawal_destination;
        let auction_house = &ctx.accounts.auction_house;
        let system_program = &ctx.accounts.system_program;

        let auction_house_key = auction_house.key();
        let seeds = [
            PREFIX.as_bytes(),
            auction_house_key.as_ref(),
            FEE_PAYER.as_bytes(),
            &[auction_house.fee_payer_bump],
        ];

        invoke_signed(
            &system_instruction::transfer(
                &auction_house_fee_account.key(),
                &fee_withdrawal_destination.key(),
                amount,
            ),
            &[
                auction_house_fee_account.to_account_info(),
                fee_withdrawal_destination.to_account_info(),
                system_program.to_account_info(),
            ],
            &[&seeds],
        )?;

        Ok(())
    }

    /// Withdraw `amount` from the Auction House Treasury Account to a provided destination account.
    pub fn withdraw_from_treasury<'info>(
        ctx: Context<'_, '_, '_, 'info, WithdrawFromTreasury<'info>>,
        amount: u64,
    ) -> ProgramResult {
        let treasury_mint = &ctx.accounts.treasury_mint;
        let treasury_withdrawal_destination = &ctx.accounts.treasury_withdrawal_destination;
        let auction_house_treasury = &ctx.accounts.auction_house_treasury;
        let auction_house = &ctx.accounts.auction_house;
        let token_program = &ctx.accounts.token_program;
        let system_program = &ctx.accounts.system_program;

        let is_native = treasury_mint.key() == spl_token::native_mint::id();
        let auction_house_seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref(),
            &[auction_house.bump],
        ];

        let ah_key = auction_house.key();
        let auction_house_treasury_seeds = [
            PREFIX.as_bytes(),
            ah_key.as_ref(),
            TREASURY.as_bytes(),
            &[auction_house.treasury_bump],
        ];
        if !is_native {
            invoke_signed(
                &spl_token::instruction::transfer(
                    token_program.key,
                    &auction_house_treasury.key(),
                    &treasury_withdrawal_destination.key(),
                    &auction_house.key(),
                    &[],
                    amount,
                )?,
                &[
                    auction_house_treasury.to_account_info(),
                    treasury_withdrawal_destination.to_account_info(),
                    token_program.to_account_info(),
                    auction_house.to_account_info(),
                ],
                &[&auction_house_seeds],
            )?;
        } else {
            invoke_signed(
                &system_instruction::transfer(
                    &auction_house_treasury.key(),
                    &treasury_withdrawal_destination.key(),
                    amount,
                ),
                &[
                    auction_house_treasury.to_account_info(),
                    treasury_withdrawal_destination.to_account_info(),
                    system_program.to_account_info(),
                ],
                &[&auction_house_treasury_seeds],
            )?;
        }

        Ok(())
    }

    /// Update Auction House values such as seller fee basis points, update authority, treasury account, etc.
    pub fn update_auction_house<'info>(
        ctx: Context<'_, '_, '_, 'info, UpdateAuctionHouse<'info>>,
        seller_fee_basis_points: Option<u16>,
        requires_sign_off: Option<bool>,
        can_change_sale_price: Option<bool>,
    ) -> ProgramResult {
        let treasury_mint = &ctx.accounts.treasury_mint;
        let payer = &ctx.accounts.payer;
        let new_authority = &ctx.accounts.new_authority;
        let auction_house = &mut ctx.accounts.auction_house;
        let fee_withdrawal_destination = &ctx.accounts.fee_withdrawal_destination;
        let treasury_withdrawal_destination_owner =
            &ctx.accounts.treasury_withdrawal_destination_owner;
        let treasury_withdrawal_destination = &ctx.accounts.treasury_withdrawal_destination;
        let token_program = &ctx.accounts.token_program;
        let system_program = &ctx.accounts.system_program;
        let ata_program = &ctx.accounts.ata_program;
        let rent = &ctx.accounts.rent;
        let is_native = treasury_mint.key() == spl_token::native_mint::id();

        if let Some(sfbp) = seller_fee_basis_points {
            if sfbp > 10000 {
                return Err(ErrorCode::InvalidBasisPoints.into());
            }

            auction_house.seller_fee_basis_points = sfbp;
        }

        if let Some(rqf) = requires_sign_off {
            auction_house.requires_sign_off = rqf;
        }
        if let Some(chsp) = can_change_sale_price {
            auction_house.can_change_sale_price = chsp;
        }

        auction_house.authority = new_authority.key();
        auction_house.treasury_withdrawal_destination = treasury_withdrawal_destination.key();
        auction_house.fee_withdrawal_destination = fee_withdrawal_destination.key();

        if !is_native {
            if treasury_withdrawal_destination.data_is_empty() {
                make_ata(
                    treasury_withdrawal_destination.to_account_info(),
                    treasury_withdrawal_destination_owner.to_account_info(),
                    treasury_mint.to_account_info(),
                    payer.to_account_info(),
                    ata_program.to_account_info(),
                    token_program.to_account_info(),
                    system_program.to_account_info(),
                    rent.to_account_info(),
                    &[],
                )?;
            }

            assert_is_ata(
                &treasury_withdrawal_destination.to_account_info(),
                &treasury_withdrawal_destination_owner.key(),
                &treasury_mint.key(),
            )?;
        } else {
            assert_keys_equal(
                treasury_withdrawal_destination.key(),
                treasury_withdrawal_destination_owner.key(),
            )?;
        }

        Ok(())
    }

    /// Create a new Auction House instance.
    pub fn create_auction_house<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateAuctionHouse<'info>>,
        bump: u8,
        fee_payer_bump: u8,
        treasury_bump: u8,
        seller_fee_basis_points: u16,
        requires_sign_off: bool,
        can_change_sale_price: bool,
    ) -> ProgramResult {
        let treasury_mint = &ctx.accounts.treasury_mint;
        let payer = &ctx.accounts.payer;
        let authority = &ctx.accounts.authority;
        let auction_house = &mut ctx.accounts.auction_house;
        let auction_house_fee_account = &ctx.accounts.auction_house_fee_account;
        let auction_house_treasury = &ctx.accounts.auction_house_treasury;
        let fee_withdrawal_destination = &ctx.accounts.fee_withdrawal_destination;
        let treasury_withdrawal_destination_owner =
            &ctx.accounts.treasury_withdrawal_destination_owner;
        let treasury_withdrawal_destination = &ctx.accounts.treasury_withdrawal_destination;
        let token_program = &ctx.accounts.token_program;
        let system_program = &ctx.accounts.system_program;
        let ata_program = &ctx.accounts.ata_program;
        let rent = &ctx.accounts.rent;

        auction_house.bump = bump;
        auction_house.fee_payer_bump = fee_payer_bump;
        auction_house.treasury_bump = treasury_bump;
        if seller_fee_basis_points > 10000 {
            return Err(ErrorCode::InvalidBasisPoints.into());
        }
        auction_house.seller_fee_basis_points = seller_fee_basis_points;
        auction_house.requires_sign_off = requires_sign_off;
        auction_house.can_change_sale_price = can_change_sale_price;
        auction_house.creator = authority.key();
        auction_house.authority = authority.key();
        auction_house.treasury_mint = treasury_mint.key();
        auction_house.auction_house_fee_account = auction_house_fee_account.key();
        auction_house.auction_house_treasury = auction_house_treasury.key();
        auction_house.treasury_withdrawal_destination = treasury_withdrawal_destination.key();
        auction_house.fee_withdrawal_destination = fee_withdrawal_destination.key();
        auction_house.has_auctioneer = false;

        let is_native = treasury_mint.key() == spl_token::native_mint::id();

        let ah_key = auction_house.key();

        let auction_house_treasury_seeds = [
            PREFIX.as_bytes(),
            ah_key.as_ref(),
            TREASURY.as_bytes(),
            &[treasury_bump],
        ];

        create_program_token_account_if_not_present(
            auction_house_treasury,
            system_program,
            &payer,
            token_program,
            treasury_mint,
            &auction_house.to_account_info(),
            rent,
            &auction_house_treasury_seeds,
            &[],
            is_native,
        )?;

        if !is_native {
            if treasury_withdrawal_destination.data_is_empty() {
                make_ata(
                    treasury_withdrawal_destination.to_account_info(),
                    treasury_withdrawal_destination_owner.to_account_info(),
                    treasury_mint.to_account_info(),
                    payer.to_account_info(),
                    ata_program.to_account_info(),
                    token_program.to_account_info(),
                    system_program.to_account_info(),
                    rent.to_account_info(),
                    &[],
                )?;
            }

            assert_is_ata(
                &treasury_withdrawal_destination.to_account_info(),
                &treasury_withdrawal_destination_owner.key(),
                &treasury_mint.key(),
            )?;
        } else {
            assert_keys_equal(
                treasury_withdrawal_destination.key(),
                treasury_withdrawal_destination_owner.key(),
            )?;
        }

        Ok(())
    }

    /// Deposit `amount` into the escrow payment account for your specific wallet.
    pub fn deposit<'info>(
        ctx: Context<'_, '_, '_, 'info, Deposit<'info>>,
        escrow_payment_bump: u8,
        amount: u64,
    ) -> ProgramResult {
        deposit::deposit(ctx, escrow_payment_bump, amount)
    }

    /// Deposit `amount` into the escrow payment account for your specific wallet.
    pub fn deposit_with_auctioneer<'info>(
        ctx: Context<'_, '_, '_, 'info, DepositWithAuctioneer<'info>>,
        escrow_payment_bump: u8,
        ah_auctioneer_pda_bump: u8,
        amount: u64,
    ) -> ProgramResult {
        deposit::deposit_with_auctioneer(ctx, escrow_payment_bump, ah_auctioneer_pda_bump, amount)
    }

    pub fn cancel<'info>(
        ctx: Context<'_, '_, '_, 'info, Cancel<'info>>,
        buyer_price: u64,
        token_size: u64,
    ) -> ProgramResult {
        cancel::cancel(ctx, buyer_price, token_size)
    }

    pub fn cancel_with_auctioneer<'info>(
        ctx: Context<'_, '_, '_, 'info, CancelWithAuctioneer<'info>>,
        ah_auctioneer_pda_bump: u8,
        buyer_price: u64,
        token_size: u64,
    ) -> ProgramResult {
        cancel::cancel_with_auctioneer(ctx, ah_auctioneer_pda_bump, buyer_price, token_size)
    }

    /// Withdraw `amount` from the escrow payment account for your specific wallet.
    pub fn withdraw<'info>(
        ctx: Context<'_, '_, '_, 'info, Withdraw<'info>>,
        escrow_payment_bump: u8,
        amount: u64,
    ) -> ProgramResult {
        withdraw::withdraw(ctx, escrow_payment_bump, amount)
    }

    /// Withdraw `amount` from the escrow payment account for your specific wallet.
    pub fn withdraw_with_auctioneer<'info>(
        ctx: Context<'_, '_, '_, 'info, WithdrawWithAuctioneer<'info>>,
        escrow_payment_bump: u8,
        ah_auctioneer_pda_bump: u8,
        amount: u64,
    ) -> ProgramResult {
        withdraw::withdraw_with_auctioneer(ctx, escrow_payment_bump, ah_auctioneer_pda_bump, amount)
    }

    pub fn execute_sale<'info>(
        ctx: Context<'_, '_, '_, 'info, ExecuteSale<'info>>,
        escrow_payment_bump: u8,
        _free_trade_state_bump: u8,
        program_as_signer_bump: u8,
        buyer_price: u64,
        token_size: u64,
    ) -> ProgramResult {
        execute_sale::execute_sale(
            ctx,
            escrow_payment_bump,
            _free_trade_state_bump,
            program_as_signer_bump,
            buyer_price,
            token_size,
        )
    }

    pub fn execute_sale_with_auctioneer<'info>(
        ctx: Context<'_, '_, '_, 'info, ExecuteSaleWithAuctioneer<'info>>,
        escrow_payment_bump: u8,
        _free_trade_state_bump: u8,
        program_as_signer_bump: u8,
        buyer_price: u64,
        token_size: u64,
        ah_auctioneer_pda_bump: u8,
    ) -> ProgramResult {
        execute_sale::execute_sale_with_auctioneer(
            ctx,
            escrow_payment_bump,
            _free_trade_state_bump,
            program_as_signer_bump,
            buyer_price,
            token_size,
            ah_auctioneer_pda_bump,
        )
    }

    pub fn sell<'info>(
        ctx: Context<'_, '_, '_, 'info, Sell<'info>>,
        trade_state_bump: u8,
        free_trade_state_bump: u8,
        program_as_signer_bump: u8,
        buyer_price: u64,
        token_size: u64,
    ) -> ProgramResult {
        sell::sell(
            ctx,
            trade_state_bump,
            free_trade_state_bump,
            program_as_signer_bump,
            buyer_price,
            token_size,
        )
    }

    pub fn sell_with_auctioneer<'info>(
        ctx: Context<'_, '_, '_, 'info, SellWithAuctioneer<'info>>,
        trade_state_bump: u8,
        free_trade_state_bump: u8,
        program_as_signer_bump: u8,
        buyer_price: u64,
        token_size: u64,
        ah_auctioneer_pda_bump: u8,
    ) -> ProgramResult {
        sell::sell_with_auctioneer(
            ctx,
            trade_state_bump,
            free_trade_state_bump,
            program_as_signer_bump,
            buyer_price,
            token_size,
            ah_auctioneer_pda_bump,
        )
    }

    /// Create a private buy bid by creating a `buyer_trade_state` account and an `escrow_payment` account and funding the escrow with the necessary SOL or SPL token amount.
    pub fn buy<'info>(
        ctx: Context<'_, '_, '_, 'info, Buy<'info>>,
        trade_state_bump: u8,
        escrow_payment_bump: u8,
        buyer_price: u64,
        token_size: u64,
    ) -> ProgramResult {
        private_bid(
            ctx,
            trade_state_bump,
            escrow_payment_bump,
            buyer_price,
            token_size,
        )
    }

    pub fn buy_with_auctioneer<'info>(
        ctx: Context<'_, '_, '_, 'info, BuyWithAuctioneer<'info>>,
        trade_state_bump: u8,
        escrow_payment_bump: u8,
        buyer_price: u64,
        token_size: u64,
        ah_auctioneer_pda_bump: u8,
    ) -> ProgramResult {
        private_bid_with_auctioneer(
            ctx,
            trade_state_bump,
            escrow_payment_bump,
            buyer_price,
            token_size,
            ah_auctioneer_pda_bump,
        )
    }

    /// Create a public buy bid by creating a `public_buyer_trade_state` account and an `escrow_payment` account and funding the escrow with the necessary SOL or SPL token amount.
    pub fn public_buy<'info>(
        ctx: Context<'_, '_, '_, 'info, PublicBuy<'info>>,
        trade_state_bump: u8,
        escrow_payment_bump: u8,
        buyer_price: u64,
        token_size: u64,
    ) -> ProgramResult {
        public_bid(
            ctx,
            trade_state_bump,
            escrow_payment_bump,
            buyer_price,
            token_size,
        )
    }

    /// Create a public buy bid by creating a `public_buyer_trade_state` account and an `escrow_payment` account and funding the escrow with the necessary SOL or SPL token amount.
    pub fn public_buy_with_auctioneer<'info>(
        ctx: Context<'_, '_, '_, 'info, PublicBuyWithAuctioneer<'info>>,
        trade_state_bump: u8,
        escrow_payment_bump: u8,
        buyer_price: u64,
        token_size: u64,
        ah_auctioneer_pda_bump: u8,
    ) -> ProgramResult {
        public_bid_with_auctioneer(
            ctx,
            trade_state_bump,
            escrow_payment_bump,
            buyer_price,
            token_size,
            ah_auctioneer_pda_bump,
        )
    }

    pub fn delegate_auctioneer<'info>(
        ctx: Context<'_, '_, '_, 'info, DelegateAuctioneer<'info>>,
        ah_auctioneer_pda_bump: u8,
        scopes: Vec<AuthorityScope>,
    ) -> ProgramResult {
        delegate::delegate_auctioneer(ctx, ah_auctioneer_pda_bump, Box::new(scopes))
    }

    /// Create a listing receipt by creating a `listing_receipt` account.
    pub fn print_listing_receipt<'info>(
        ctx: Context<'_, '_, '_, 'info, PrintListingReceipt<'info>>,
        receipt_bump: u8,
    ) -> ProgramResult {
        receipt::print_listing_receipt(ctx, receipt_bump)
    }

    /// Cancel an active listing receipt by setting the `canceled_at` field to the current time.
    pub fn cancel_listing_receipt<'info>(
        ctx: Context<'_, '_, '_, 'info, CancelListingReceipt<'info>>,
    ) -> ProgramResult {
        receipt::cancel_listing_receipt(ctx)
    }

    /// Create a bid receipt by creating a `bid_receipt` account.
    pub fn print_bid_receipt<'info>(
        ctx: Context<'_, '_, '_, 'info, PrintBidReceipt<'info>>,
        receipt_bump: u8,
    ) -> ProgramResult {
        receipt::print_bid_receipt(ctx, receipt_bump)
    }

    /// Cancel an active bid receipt by setting the `canceled_at` field to the current time.
    pub fn cancel_bid_receipt<'info>(
        ctx: Context<'_, '_, '_, 'info, CancelBidReceipt<'info>>,
    ) -> ProgramResult {
        receipt::cancel_bid_receipt(ctx)
    }

    /// Create a purchase receipt by creating a `purchase_receipt` account.
    pub fn print_purchase_receipt<'info>(
        ctx: Context<'_, '_, '_, 'info, PrintPurchaseReceipt<'info>>,
        purchase_receipt_bump: u8,
    ) -> ProgramResult {
        receipt::print_purchase_receipt(ctx, purchase_receipt_bump)
    }
}

/// Accounts for the [`create_auction_house` handler](auction_house/fn.create_auction_house.html).
#[derive(Accounts)]
#[instruction(bump: u8, fee_payer_bump: u8, treasury_bump: u8)]
pub struct CreateAuctionHouse<'info> {
    /// Treasury mint account, either native SOL mint or a SPL token mint.
    pub treasury_mint: Account<'info, Mint>,
    /// Key paying SOL fees for setting up the Auction House.
    pub payer: Signer<'info>,
    // Authority key for the Auction House.
    pub authority: UncheckedAccount<'info>,
    /// Account that pays for fees if the marketplace executes sales.
    #[account(mut)]
    pub fee_withdrawal_destination: UncheckedAccount<'info>,
    /// SOL or SPL token account to receive Auction House fees. If treasury mint is native this will be the same as the `treasury_withdrawl_destination_owner`.
    #[account(mut)]
    pub treasury_withdrawal_destination: UncheckedAccount<'info>,
    /// Owner of the `treasury_withdrawal_destination` account or the same address if the `treasury_mint` is native.
    pub treasury_withdrawal_destination_owner: UncheckedAccount<'info>,
    /// Auction House instance PDA account.
    #[account(init, seeds=[PREFIX.as_bytes(), authority.key().as_ref(), treasury_mint.key().as_ref()], bump=bump, space=AUCTION_HOUSE_SIZE, payer=payer)]
    pub auction_house: Account<'info, AuctionHouse>,
    /// Auction House instance fee account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], bump=fee_payer_bump)]
    pub auction_house_fee_account: UncheckedAccount<'info>,
    /// Auction House instance treasury PDA account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), TREASURY.as_bytes()], bump=treasury_bump)]
    pub auction_house_treasury: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub ata_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

/// Accounts for the [`update_auction_house` handler](auction_house/fn.update_auction_house.html).
#[derive(Accounts)]
pub struct UpdateAuctionHouse<'info> {
    /// Treasury mint account, either native SOL mint or a SPL token mint.
    pub treasury_mint: Account<'info, Mint>,
    /// Key paying SOL fees for setting up the Auction House.
    pub payer: Signer<'info>,
    /// Authority key for the Auction House.
    pub authority: Signer<'info>,
    /// New authority key for the Auction House.
    pub new_authority: UncheckedAccount<'info>,
    /// Account that pays for fees if the marketplace executes sales.
    #[account(mut)]
    pub fee_withdrawal_destination: UncheckedAccount<'info>,
    /// SOL or SPL token account to receive Auction House fees. If treasury mint is native this will be the same as the `treasury_withdrawl_destination_owner`.
    #[account(mut)]
    pub treasury_withdrawal_destination: UncheckedAccount<'info>,
    /// Owner of the `treasury_withdrawal_destination` account or the same address if the `treasury_mint` is native.
    pub treasury_withdrawal_destination_owner: UncheckedAccount<'info>,
    /// Auction House instance PDA account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), treasury_mint.key().as_ref()], bump=auction_house.bump, has_one=authority, has_one=treasury_mint)]
    pub auction_house: Account<'info, AuctionHouse>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub ata_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

/// Accounts for the [`withdraw_from_treasury` handler](auction_house/fn.withdraw_from_treasury.html).
#[derive(Accounts)]
pub struct WithdrawFromTreasury<'info> {
    /// Treasury mint account, either native SOL mint or a SPL token mint.
    pub treasury_mint: Account<'info, Mint>,
    /// Authority key for the Auction House.
    pub authority: Signer<'info>,
    /// SOL or SPL token account to receive Auction House fees. If treasury mint is native this will be the same as the `treasury_withdrawl_destination_owner`.
    #[account(mut)]
    pub treasury_withdrawal_destination: UncheckedAccount<'info>,
    /// Auction House treasury PDA account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), TREASURY.as_bytes()], bump=auction_house.treasury_bump)]
    pub auction_house_treasury: UncheckedAccount<'info>,
    /// Auction House instance PDA account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), treasury_mint.key().as_ref()], bump=auction_house.bump, has_one=authority, has_one=treasury_mint, has_one=treasury_withdrawal_destination, has_one=auction_house_treasury)]
    pub auction_house: Account<'info, AuctionHouse>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Accounts for the [`withdraw_from_fee` handler](auction_house/fn.withdraw_from_fee.html).
#[derive(Accounts)]
pub struct WithdrawFromFee<'info> {
    /// Authority key for the Auction House.
    pub authority: Signer<'info>,
    /// Account that pays for fees if the marketplace executes sales.
    #[account(mut)]
    pub fee_withdrawal_destination: UncheckedAccount<'info>,
    /// Auction House instance fee account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], bump=auction_house.fee_payer_bump)]
    pub auction_house_fee_account: UncheckedAccount<'info>,
    /// Auction House instance PDA account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.key().as_ref()], bump=auction_house.bump, has_one=authority, has_one=fee_withdrawal_destination, has_one=auction_house_fee_account)]
    pub auction_house: Account<'info, AuctionHouse>,
    pub system_program: Program<'info, System>,
}
