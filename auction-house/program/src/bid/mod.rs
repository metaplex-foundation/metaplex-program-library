use anchor_lang::{
    AnchorDeserialize,
    AnchorSerialize,
    prelude::*, solana_program::{
        program::invoke,
        system_instruction,
    },
};
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{AuctionHouse, ErrorCode, TRADE_STATE_SIZE};
use crate::constants::*;
use crate::utils::*;

#[derive(Accounts)]
#[instruction(trade_state_bump: u8, escrow_payment_bump: u8, buyer_price: u64, token_size: u64)]
pub struct PublicBuy<'info> {
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

pub fn public_bid(
    ctx: Context<PublicBuy>,
    trade_state_bump: u8,
    escrow_payment_bump: u8,
    buyer_price: u64,
    token_size: u64,
) -> ProgramResult {
    bid_logic(
        &ctx.accounts.wallet,
        &ctx.accounts.payment_account,
        &ctx.accounts.transfer_authority,
        &ctx.accounts.treasury_mint,
        &ctx.accounts.token_account,
        &ctx.accounts.metadata,
        &ctx.accounts.escrow_payment_account,
        &ctx.accounts.authority,
        &ctx.accounts.auction_house,
        &ctx.accounts.auction_house_fee_account,
        &mut ctx.accounts.buyer_trade_state,
        &ctx.accounts.token_program,
        &ctx.accounts.system_program,
        &ctx.accounts.rent,
        trade_state_bump,
        escrow_payment_bump,
        buyer_price,
        token_size,
        true,
    )
}

#[derive(Accounts)]
#[instruction(trade_state_bump: u8, escrow_payment_bump: u8, buyer_price: u64, token_size: u64)]
pub struct Buy<'info> {
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

pub fn private_bid<'info>(ctx: Context<'_, '_, '_, 'info, Buy<'info>>,
                          trade_state_bump: u8,
                          escrow_payment_bump: u8,
                          buyer_price: u64,
                          token_size: u64) -> ProgramResult {
    bid_logic(
        &ctx.accounts.wallet,
        &ctx.accounts.payment_account,
        &ctx.accounts.transfer_authority,
        &ctx.accounts.treasury_mint,
        &ctx.accounts.token_account,
        &ctx.accounts.metadata,
        &ctx.accounts.escrow_payment_account,
        &ctx.accounts.authority,
        &ctx.accounts.auction_house,
        &ctx.accounts.auction_house_fee_account,
        &mut ctx.accounts.buyer_trade_state,
        &ctx.accounts.token_program,
        &ctx.accounts.system_program,
        &ctx.accounts.rent,
        trade_state_bump,
        escrow_payment_bump,
        buyer_price,
        token_size,
        false,
    )
}


pub fn bid_logic<'info>(
    wallet: &Signer<'info>,
    payment_account: &UncheckedAccount<'info>,
    transfer_authority: &UncheckedAccount<'info>,
    treasury_mint: &Account<'info, Mint>,
    token_account: &Account<'info, TokenAccount>,
    metadata: &UncheckedAccount<'info>,
    escrow_payment_account: &UncheckedAccount<'info>,
    authority: &UncheckedAccount<'info>,
    auction_house: &Account<'info, AuctionHouse>,
    auction_house_fee_account: &UncheckedAccount<'info>,
    buyer_trade_state: &UncheckedAccount<'info>,
    token_program: &Program<'info, Token>,
    system_program: &Program<'info, System>,
    rent: &Sysvar<'info, Rent>,
    trade_state_bump: u8,
    escrow_payment_bump: u8,
    buyer_price: u64,
    token_size: u64,
    public: bool,
) -> ProgramResult {
    assert_valid_trade_state(
        &wallet.key(),
        &auction_house,
        buyer_price,
        token_size,
        &buyer_trade_state,
        &token_account.mint.key(),
        &token_account.key(),
        trade_state_bump,
    )?;

    let auction_house_key = auction_house.key();
    let seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        FEE_PAYER.as_bytes(),
        &[auction_house.fee_payer_bump],
    ];

    let (fee_payer, fee_seeds) = get_fee_payer(
        &authority,
        &auction_house,
        wallet.to_account_info(),
        auction_house_fee_account.to_account_info(),
        &seeds,
    )?;

    let is_native = treasury_mint.key() == spl_token::native_mint::id();

    let auction_house_key = auction_house.key();
    let wallet_key = wallet.key();
    let escrow_signer_seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        wallet_key.as_ref(),
        &[escrow_payment_bump],
    ];
    create_program_token_account_if_not_present(
        &escrow_payment_account,
        &system_program,
        &fee_payer,
        &token_program,
        &treasury_mint,
        &auction_house.to_account_info(),
        &rent,
        &escrow_signer_seeds,
        fee_seeds,
        is_native,
    )?;

    if is_native {
        assert_keys_equal(wallet.key(), payment_account.key())?;

        if escrow_payment_account.lamports() < buyer_price {
            let diff = buyer_price
                .checked_sub(escrow_payment_account.lamports())
                .ok_or(ErrorCode::NumericalOverflow)?;
            invoke(
                &system_instruction::transfer(
                    &payment_account.key(),
                    &escrow_payment_account.key(),
                    diff,
                ),
                &[
                    payment_account.to_account_info(),
                    escrow_payment_account.to_account_info(),
                    system_program.to_account_info(),
                ],
            )?;
        }
    } else {
        let escrow_payment_loaded: spl_token::state::Account =
            assert_initialized(&escrow_payment_account)?;

        if escrow_payment_loaded.amount < buyer_price {
            let diff = buyer_price
                .checked_sub(escrow_payment_loaded.amount)
                .ok_or(ErrorCode::NumericalOverflow)?;
            invoke(
                &spl_token::instruction::transfer(
                    &token_program.key(),
                    &payment_account.key(),
                    &escrow_payment_account.key(),
                    &transfer_authority.key(),
                    &[],
                    diff,
                )?,
                &[
                    transfer_authority.to_account_info(),
                    payment_account.to_account_info(),
                    escrow_payment_account.to_account_info(),
                    token_program.to_account_info(),
                ],
            )?;
        }
    }

    assert_metadata_valid(&metadata, &token_account)?;

    let ts_info = buyer_trade_state.to_account_info();
    if ts_info.data_is_empty() {
        msg!("hello 2");
        let wallet_key = wallet.key();
        let token_account_key = token_account.key();
        if public {
            let ts_seeds = [
                PREFIX.as_bytes(),
                wallet_key.as_ref(),
                auction_house_key.as_ref(),
                auction_house.treasury_mint.as_ref(),
                token_account.mint.as_ref(),
                &buyer_price.to_le_bytes(),
                &token_size.to_le_bytes(),
                &[trade_state_bump],
            ];
            create_or_allocate_account_raw(
                crate::id(),
                &ts_info,
                &rent.to_account_info(),
                &system_program,
                &fee_payer,
                TRADE_STATE_SIZE,
                fee_seeds,
                &ts_seeds,
            )?;
        } else {
            let ts_seeds = [
                PREFIX.as_bytes(),
                wallet_key.as_ref(),
                auction_house_key.as_ref(),
                token_account_key.as_ref(),
                auction_house.treasury_mint.as_ref(),
                token_account.mint.as_ref(),
                &buyer_price.to_le_bytes(),
                &token_size.to_le_bytes(),
                &[trade_state_bump],
            ];
            create_or_allocate_account_raw(
                crate::id(),
                &ts_info,
                &rent.to_account_info(),
                &system_program,
                &fee_payer,
                TRADE_STATE_SIZE,
                fee_seeds,
                &ts_seeds,
            )?;
        }
    }
    let data = &mut ts_info.data.borrow_mut();
    data[0] = trade_state_bump;

    Ok(())
}
