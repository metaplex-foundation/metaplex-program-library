use anchor_lang::{prelude::*, solana_program::program::invoke, AnchorDeserialize};
use solana_program::program_memory::sol_memset;

use crate::{constants::*, errors::*, utils::*, AuctionHouse, AuthorityScope, *};

/// Accounts for the [`cancel` handler](auction_house/fn.cancel.html).
#[derive(Accounts)]
#[instruction(buyer_price: u64, token_size: u64)]
pub struct InstantCancel<'info> {
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

impl<'info> From<CancelWithAuctioneer<'info>> for InstantCancel<'info> {
    fn from(a: CancelWithAuctioneer<'info>) -> InstantCancel<'info> {
        InstantCancel {
            wallet: a.wallet,
            token_account: a.token_account,
            token_mint: a.token_mint,
            authority: a.authority,
            auction_house: a.auction_house,
            auction_house_fee_account: a.auction_house_fee_account,
            trade_state: a.trade_state,
            token_program: a.token_program,
        }
    }
}

/// Accounts for the [`cancel` handler](auction_house/fn.cancel.html).
#[derive(Accounts, Clone)]
#[instruction(auctioneer_pda_bump: u8, buyer_price: u64, token_size: u64)]
pub struct CancelWithAuctioneer<'info> {
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

    /// The auctioneer program PDA running this auction.
    pub auctioneer_authority: UncheckedAccount<'info>,

    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(seeds = [AUCTIONEER.as_bytes(), auction_house.key().as_ref(), auctioneer_authority.key().as_ref()], bump = auctioneer_pda_bump)]
    pub ah_auctioneer_pda: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

// Cancel a bid or ask by revoking the token delegate, transferring all lamports from the trade state account to the fee payer, and setting the trade state account data to zero so it can be garbage collected.
pub fn instant_cancel<'info>(
    mut ctx: Context<'_, '_, '_, 'info, InstantCancel<'info>>,
    buyer_price: u64,
    token_size: u64,
) -> ProgramResult {
    let auction_house = &ctx.accounts.auction_house;

    // If it has an auctioneer authority delegated must use *_with_auctioneer handler.
    if auction_house.has_auctioneer {
        return Err(ErrorCode::MustUseAuctioneerHandler.into());
    }

    cancel(&mut ctx.accounts, buyer_price, token_size)
}

pub fn cancel_with_auctioneer<'info>(
    ctx: Context<'_, '_, '_, 'info, CancelWithAuctioneer<'info>>,
    _auctioneer_pda_bump: u8,
    buyer_price: u64,
    token_size: u64,
) -> ProgramResult {
    let auction_house = &ctx.accounts.auction_house;
    let auctioneer_authority = &ctx.accounts.auctioneer_authority;
    let ah_auctioneer_pda = &ctx.accounts.ah_auctioneer_pda;

    if !auction_house.has_auctioneer {
        return Err(ErrorCode::NoAuctioneerProgramSet.into());
    }

    assert_valid_auctioneer_and_scope(
        &auction_house.key(),
        &auctioneer_authority.key(),
        ah_auctioneer_pda,
        AuthorityScope::Cancel,
    )?;

    let mut accounts: InstantCancel<'info> = (*ctx.accounts).clone().into();

    cancel(&mut accounts, buyer_price, token_size)
}

fn cancel<'info>(
    accounts: &mut InstantCancel<'info>,
    buyer_price: u64,
    token_size: u64,
) -> ProgramResult {
    let wallet = &accounts.wallet;
    let token_account = &accounts.token_account;
    let token_mint = &accounts.token_mint;
    let authority = &accounts.authority;
    let auction_house = &accounts.auction_house;
    let auction_house_fee_account = &accounts.auction_house_fee_account;
    let trade_state = &accounts.trade_state;
    let token_program = &accounts.token_program;

    let ts_bump = trade_state.try_borrow_data()?[0];
    assert_valid_trade_state(
        &wallet.key(),
        auction_house,
        buyer_price,
        token_size,
        &trade_state.to_account_info(),
        &token_account.mint.key(),
        &token_account.key(),
        ts_bump,
    )?;
    assert_keys_equal(token_mint.key(), token_account.mint)?;
    if !wallet.to_account_info().is_signer && !authority.to_account_info().is_signer {
        return Err(ErrorCode::NoValidSignerPresent.into());
    }

    let auction_house_key = auction_house.key();
    let seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        FEE_PAYER.as_bytes(),
        &[auction_house.fee_payer_bump],
    ];

    let (fee_payer, _) = get_fee_payer(
        authority,
        auction_house,
        wallet.to_account_info(),
        auction_house_fee_account.to_account_info(),
        &seeds,
    )?;

    if token_account.owner == wallet.key() && wallet.is_signer {
        invoke(
            &revoke(
                &token_program.key(),
                &token_account.key(),
                &wallet.key(),
                &[],
            )
            .unwrap(),
            &[
                token_program.to_account_info(),
                token_account.to_account_info(),
                wallet.to_account_info(),
            ],
        )?;
    }

    let curr_lamp = trade_state.lamports();
    **trade_state.lamports.borrow_mut() = 0;

    **fee_payer.lamports.borrow_mut() = fee_payer
        .lamports()
        .checked_add(curr_lamp)
        .ok_or(ErrorCode::NumericalOverflow)?;
    sol_memset(*trade_state.try_borrow_mut_data()?, 0, TRADE_STATE_SIZE);
    Ok(())
}
