use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke, system_instruction},
};
use anchor_spl::token::{Mint, Token, TokenAccount};
use solana_program::program_memory::sol_memset;

use crate::{
    constants::*, errors::AuctionHouseError, utils::*, AuctionHouse, Auctioneer, AuthorityScope,
    TRADE_STATE_SIZE,
};

// Handles the bid logic for both private and public auctioneer bids.
#[allow(clippy::too_many_arguments)]
pub fn auctioneer_bid_logic<'info>(
    wallet: Signer<'info>,
    payment_account: UncheckedAccount<'info>,
    transfer_authority: UncheckedAccount<'info>,
    treasury_mint: Account<'info, Mint>,
    token_account: Account<'info, TokenAccount>,
    metadata: UncheckedAccount<'info>,
    escrow_payment_account: UncheckedAccount<'info>,
    auction_house: &mut Box<Account<'info, AuctionHouse>>,
    auction_house_fee_account: UncheckedAccount<'info>,
    buyer_trade_state: UncheckedAccount<'info>,
    authority: UncheckedAccount<'info>,
    auctioneer_authority: Signer<'info>,
    ah_auctioneer_pda: Account<'info, Auctioneer>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
    trade_state_bump: u8,
    escrow_payment_bump: u8,
    buyer_price: u64,
    token_size: u64,
    public: bool,
    escrow_canonical_bump: u8,
    trade_state_canonical_bump: u8,
) -> Result<()> {
    if !auction_house.has_auctioneer {
        return Err(AuctionHouseError::NoAuctioneerProgramSet.into());
    }

    assert_valid_auctioneer_and_scope(
        auction_house,
        &auctioneer_authority.key(),
        &ah_auctioneer_pda,
        AuthorityScope::Buy,
    )?;

    if (escrow_canonical_bump != escrow_payment_bump)
        || (trade_state_canonical_bump != trade_state_bump)
    {
        return Err(AuctionHouseError::BumpSeedNotInHashMap.into());
    }

    assert_valid_trade_state(
        &wallet.key(),
        auction_house,
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
        auction_house,
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

        if escrow_payment_account.lamports()
            < buyer_price
                .checked_add(rent.minimum_balance(escrow_payment_account.data_len()))
                .ok_or(AuctionHouseError::NumericalOverflow)?
        {
            let diff = buyer_price
                .checked_add(rent.minimum_balance(escrow_payment_account.data_len()))
                .ok_or(AuctionHouseError::NumericalOverflow)?
                .checked_sub(escrow_payment_account.lamports())
                .ok_or(AuctionHouseError::NumericalOverflow)?;

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
                .ok_or(AuctionHouseError::NumericalOverflow)?;
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
        let wallet_key = wallet.key();
        let token_account_key = token_account.key();
        if public {
            create_or_allocate_account_raw(
                crate::id(),
                &ts_info,
                &rent.to_account_info(),
                &system_program,
                &fee_payer,
                TRADE_STATE_SIZE,
                fee_seeds,
                &[
                    PREFIX.as_bytes(),
                    wallet_key.as_ref(),
                    auction_house_key.as_ref(),
                    auction_house.treasury_mint.as_ref(),
                    token_account.mint.as_ref(),
                    &buyer_price.to_le_bytes(),
                    &token_size.to_le_bytes(),
                    &[trade_state_bump],
                ],
            )?;
        } else {
            create_or_allocate_account_raw(
                crate::id(),
                &ts_info,
                &rent.to_account_info(),
                &system_program,
                &fee_payer,
                TRADE_STATE_SIZE,
                fee_seeds,
                &[
                    PREFIX.as_bytes(),
                    wallet_key.as_ref(),
                    auction_house_key.as_ref(),
                    token_account_key.as_ref(),
                    auction_house.treasury_mint.as_ref(),
                    token_account.mint.as_ref(),
                    &buyer_price.to_le_bytes(),
                    &token_size.to_le_bytes(),
                    &[trade_state_bump],
                ],
            )?;
        }
        sol_memset(
            *ts_info.try_borrow_mut_data()?,
            trade_state_bump,
            TRADE_STATE_SIZE,
        );
    }
    // Allow The same bid to be sent with no issues
    Ok(())
}
