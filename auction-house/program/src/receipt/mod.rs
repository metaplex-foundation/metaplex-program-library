use anchor_lang::{prelude::*, AnchorDeserialize, AnchorSerialize};
use anchor_spl::token::TokenAccount;
use crate::{constants::*, id, utils::*, AuctionHouse, ErrorCode};

pub const PUBLIC_BID_RECEIPT_SIZE: usize = 8 + //key
32 + // trade_state
32 + // bookkeeper
32 + // auction_house
32 + // wallet
32 + // token_mint
8 + // price
8 + // token_size
1 + // bump
1 + // trade_state_bump
1 + 8 + // activated_at
1 + 8; // closed_at

#[account]
pub struct PublicBidReceipt {
    pub trade_state: Pubkey,
    pub bookkeeper: Pubkey,
    pub auction_house: Pubkey,
    pub wallet: Pubkey,
    pub token_mint: Pubkey,
    pub price: u64,
    pub token_size: u64,
    pub bump: u8,
    pub trade_state_bump: u8,
    pub activated_at: Option<i64>,
    pub closed_at: Option<i64>,
}

pub const LISTING_RECEIPT_SIZE: usize = 8 + //key
32 + // trade_state
32 + // bookkeeper
32 + // auction_house
32 + // seller
32 + // token_mint
8 + // price
8 + // token_size
1 + // bump
1 + // trade_state_bump
1 + 8 + // activated_at
1 + 8; // closed_at;

#[account]
pub struct ListingReceipt {
    pub trade_state: Pubkey,
    pub bookkeeper: Pubkey,
    pub auction_house: Pubkey,
    pub seller: Pubkey,
    pub token_mint: Pubkey,
    pub price: u64,
    pub token_size: u64,
    pub bump: u8,
    pub trade_state_bump: u8,
    pub activated_at: Option<i64>,
    pub closed_at: Option<i64>,
}

pub const PURCHASE_RECEIPT_SIZE: usize = 8 + //key
32 + // buyer
32 + // seller
32 + // auction_house
32 + // token_mint
8 + // token_size
8 + // price
1 + // bump
1 + 8; // created_at

#[account]
pub struct PurchaseReceipt {
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub auction_house: Pubkey,
    pub token_mint: Pubkey,
    pub token_size: u64,
    pub price: u64,
    pub bump: u8,
    pub created_at: Option<i64>,
}

#[derive(Accounts)]
#[instruction(trade_state_bump: u8, receipt_bump: u8, price: u64, token_size: u64)]
pub struct PrintListingReceipt<'info> {
    wallet: UncheckedAccount<'info>,
    token_account: Account<'info, TokenAccount>,
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], bump=auction_house.bump)]
    auction_house: Account<'info, AuctionHouse>,
    #[account(seeds=[PREFIX.as_bytes(), wallet.key().as_ref(), auction_house.key().as_ref(), token_account.key().as_ref(), auction_house.treasury_mint.as_ref(), token_account.mint.as_ref(), &price.to_le_bytes(), &token_size.to_le_bytes()], bump=trade_state_bump)]
    trade_state: UncheckedAccount<'info>,
    #[account(init, seeds=[LISTING_RECEIPT_PREFIX.as_bytes(), trade_state.key().as_ref()], bump=receipt_bump, payer=bookkeeper, space=LISTING_RECEIPT_SIZE)]
    receipt: Account<'info, ListingReceipt>,
    #[account(mut)]
    bookkeeper: Signer<'info>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
    clock: Sysvar<'info, Clock>,
}

pub fn print_listing_receipt<'info>(
    ctx: Context<'_, '_, '_, 'info, PrintListingReceipt<'info>>,
    trade_state_bump: u8,
    receipt_bump: u8,
    price: u64,
    token_size: u64,
) -> ProgramResult {
    let trade_state = &ctx.accounts.trade_state;
    let receipt = &mut ctx.accounts.receipt;
    let auction_house = &ctx.accounts.auction_house;
    let token_account = &ctx.accounts.token_account;
    let bookkeeper = &ctx.accounts.bookkeeper;
    let wallet = &ctx.accounts.wallet;
    let clock = &ctx.accounts.clock;

    let token_account_key = token_account.key();

    receipt.trade_state = trade_state.key();
    receipt.auction_house = auction_house.key();
    receipt.token_mint = token_account.mint.key();
    receipt.bookkeeper = bookkeeper.key();
    receipt.seller = wallet.key();
    receipt.price = price;
    receipt.token_size = token_size;
    receipt.bump = receipt_bump;
    receipt.trade_state_bump = trade_state_bump;

    if receipt.activated_at.is_none() || receipt.closed_at.is_some() {
        receipt.activated_at = Some(clock.unix_timestamp);
        receipt.closed_at = None;
    }

    let ts_seeds = [
        PREFIX.as_bytes(),
        receipt.seller.as_ref(),
        receipt.auction_house.as_ref(),
        token_account_key.as_ref(),
        auction_house.treasury_mint.as_ref(),
        receipt.token_mint.as_ref(),
        &receipt.price.to_le_bytes(),
        &receipt.token_size.to_le_bytes(),
    ];

    assert_is_ata(
        &token_account.to_account_info(),
        &receipt.seller.key(),
        &receipt.token_mint.key(),
    )?;

    assert_derivation(&id(), &trade_state.to_account_info(), &ts_seeds)?;

    if trade_state.data_is_empty() {
        return Err(ErrorCode::TradeStateDoesntExist.into());
    }

    Ok(())
}

#[derive(Accounts)]
pub struct CloseListingReceipt<'info> {
    token_account: Account<'info, TokenAccount>,
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], bump=auction_house.bump)]
    auction_house: Account<'info, AuctionHouse>,
    #[account(seeds=[PREFIX.as_bytes(), receipt.seller.key().as_ref(), receipt.auction_house.key().as_ref(), token_account.key().as_ref(), auction_house.treasury_mint.as_ref(), receipt.token_mint.as_ref(), &receipt.price.to_le_bytes(), &receipt.token_size.to_le_bytes()], bump=receipt.trade_state_bump)]
    trade_state: UncheckedAccount<'info>,
    #[account(mut, seeds=[LISTING_RECEIPT_PREFIX.as_bytes(), trade_state.key().as_ref()], bump=receipt.bump)]
    receipt: Account<'info, ListingReceipt>,
    system_program: Program<'info, System>,
    clock: Sysvar<'info, Clock>,
}

pub fn close_listing_receipt<'info>(
    ctx: Context<'_, '_, '_, 'info, CloseListingReceipt<'info>>,
) -> ProgramResult {
    let trade_state = &ctx.accounts.trade_state;
    let token_account = &ctx.accounts.token_account;
    let receipt = &mut ctx.accounts.receipt;
    let auction_house = &ctx.accounts.auction_house;
    let clock = &ctx.accounts.clock;

    let token_account_pubkey = token_account.key();

    let ts_seeds = [
        PREFIX.as_bytes(),
        receipt.seller.as_ref(),
        receipt.auction_house.as_ref(),
        token_account_pubkey.as_ref(),
        auction_house.treasury_mint.as_ref(),
        receipt.token_mint.as_ref(),
        &receipt.price.to_le_bytes(),
        &receipt.token_size.to_le_bytes(),
    ];

    assert_is_ata(
        &token_account.to_account_info(),
        &receipt.seller.key(),
        &receipt.token_mint.key(),
    )?;

    assert_derivation(&id(), &trade_state.to_account_info(), &ts_seeds)?;

    if !trade_state.data_is_empty() {
        return Err(ErrorCode::TradeStateIsNotEmpty.into());
    }

    if receipt.to_account_info().data_is_empty() {
        return Err(ErrorCode::ReceiptIsEmpty.into());
    }

    if receipt.closed_at == None {
        receipt.closed_at = Some(clock.unix_timestamp);
    }

    Ok(())
}

#[derive(Accounts)]
#[instruction(trade_state_bump: u8, receipt_bump: u8, price: u64, token_size: u64)]
pub struct PrintPublicBidReceipt<'info> {
    wallet: UncheckedAccount<'info>,
    token_account: Account<'info, TokenAccount>,
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], bump=auction_house.bump)]
    auction_house: Account<'info, AuctionHouse>,
    #[account(seeds=[PREFIX.as_bytes(), wallet.key().as_ref(), auction_house.key().as_ref(), auction_house.treasury_mint.as_ref(), token_account.mint.as_ref(), &price.to_le_bytes(), &token_size.to_le_bytes()], bump=trade_state_bump)]
    trade_state: UncheckedAccount<'info>,
    #[account(init, seeds=[PUBLIC_BID_RECEIPT_PREFIX.as_bytes(), trade_state.key().as_ref()], bump=receipt_bump, payer=bookkeeper, space=PUBLIC_BID_RECEIPT_SIZE)]
    receipt: Account<'info, PublicBidReceipt>,
    #[account(mut)]
    bookkeeper: Signer<'info>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
    clock: Sysvar<'info, Clock>,
}

pub fn print_public_bid_receipt<'info>(
    ctx: Context<'_, '_, '_, 'info, PrintPublicBidReceipt<'info>>,
    trade_state_bump: u8,
    receipt_bump: u8,
    price: u64,
    token_size: u64,
) -> ProgramResult {
    let trade_state = &ctx.accounts.trade_state;
    let receipt = &mut ctx.accounts.receipt;
    let auction_house = &ctx.accounts.auction_house;
    let token_account = &ctx.accounts.token_account;
    let bookkeeper = &ctx.accounts.bookkeeper;
    let wallet = &ctx.accounts.wallet;
    let clock = &ctx.accounts.clock;

    receipt.trade_state = trade_state.key();
    receipt.auction_house = auction_house.key();
    receipt.token_mint = token_account.mint.key();
    receipt.bookkeeper = bookkeeper.key();
    receipt.wallet = wallet.key();
    receipt.price = price;
    receipt.token_size = token_size;
    receipt.bump = receipt_bump;
    receipt.trade_state_bump = trade_state_bump;

    if receipt.activated_at.is_none() || receipt.closed_at.is_some() {
        receipt.activated_at = Some(clock.unix_timestamp);
        receipt.closed_at = None;
    }

    let ts_seeds = [
        PREFIX.as_bytes(),
        receipt.wallet.as_ref(),
        receipt.auction_house.as_ref(),
        auction_house.treasury_mint.as_ref(),
        receipt.token_mint.as_ref(),
        &receipt.price.to_le_bytes(),
        &receipt.token_size.to_le_bytes(),
    ];

    assert_derivation(&id(), &trade_state.to_account_info(), &ts_seeds)?;

    if trade_state.data_is_empty() {
        return Err(ErrorCode::TradeStateDoesntExist.into());
    }

    Ok(())
}

#[derive(Accounts)]
pub struct ClosePublicBidReceipt<'info> {
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], bump=auction_house.bump)]
    auction_house: Account<'info, AuctionHouse>,
    #[account(seeds=[PREFIX.as_bytes(), receipt.wallet.key().as_ref(), receipt.auction_house.key().as_ref(), auction_house.treasury_mint.as_ref(), receipt.token_mint.as_ref(), &receipt.price.to_le_bytes(), &receipt.token_size.to_le_bytes()], bump=receipt.trade_state_bump)]
    trade_state: UncheckedAccount<'info>,
    #[account(mut, seeds=[PUBLIC_BID_RECEIPT_PREFIX.as_bytes(), trade_state.key().as_ref()], bump=receipt.bump)]
    receipt: Account<'info, PublicBidReceipt>,
    system_program: Program<'info, System>,
    clock: Sysvar<'info, Clock>,
}

pub fn close_public_bid_receipt<'info>(
    ctx: Context<'_, '_, '_, 'info, ClosePublicBidReceipt<'info>>,
) -> ProgramResult {
    let trade_state = &ctx.accounts.trade_state;
    let receipt = &mut ctx.accounts.receipt;
    let auction_house = &ctx.accounts.auction_house;
    let clock = &ctx.accounts.clock;

    let ts_seeds = [
        PREFIX.as_bytes(),
        receipt.wallet.as_ref(),
        receipt.auction_house.as_ref(),
        auction_house.treasury_mint.as_ref(),
        receipt.token_mint.as_ref(),
        &receipt.price.to_le_bytes(),
        &receipt.token_size.to_le_bytes(),
    ];

    assert_derivation(&id(), &trade_state.to_account_info(), &ts_seeds)?;

    if !trade_state.data_is_empty() {
        return Err(ErrorCode::TradeStateIsNotEmpty.into());
    }

    if receipt.to_account_info().data_is_empty() {
        return Err(ErrorCode::ReceiptIsEmpty.into());
    }

    if receipt.closed_at == None {
        receipt.closed_at = Some(clock.unix_timestamp);
    }

    Ok(())
}


#[derive(Accounts)]
#[instruction(receipt_bump: u8)]
pub struct PrintPurchaseReceipt<'info> {
    #[account(init, seeds=[PURCHASE_RECEIPT_PREFIX.as_bytes(), seller_trade_state.key().as_ref(), buyer_trade_state.key().as_ref()], bump=receipt_bump, payer=bookkeeper, space=PURCHASE_RECEIPT_SIZE)]
    receipt: Account<'info, PurchaseReceipt>,
    seller_trade_state: UncheckedAccount<'info>,
    buyer_trade_state: UncheckedAccount<'info>,
    #[account(mut)]
    bookkeeper: Signer<'info>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
    clock: Sysvar<'info, Clock>,
    instruction: UncheckedAccount<'info>,
}

pub fn print_purchase_receipt<'info>(
  ctx: Context<'_, '_, '_, 'info, PrintPurchaseReceipt<'info>>,
  receipt_bump: u8,
) -> ProgramResult {

let prev_instruction_data = solana_program::sysvar::instructions::get_instruction_relative(-1, &ctx.accounts.instruction)?;

  Ok(())
}