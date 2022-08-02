use anchor_lang::{prelude::*, solana_program::program::invoke};
use spl_token::instruction::approve;

use crate::{constants::*, errors::*, utils::*};

pub mod auctioneer_sell;
pub mod sell;

pub use auctioneer_sell::*;
pub use sell::*;

/// Create a sell bid by creating a `seller_trade_state` account and approving the program as the token delegate.
fn sell_logic<'info>(
    accounts: &mut Sell<'info>,
    program_id: &Pubkey,
    trade_state_bump: u8,
    _free_trade_state_bump: u8,
    _program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    let wallet = &accounts.wallet;
    let token_account = &accounts.token_account;
    let metadata = &accounts.metadata;
    let authority = &accounts.authority;
    let seller_trade_state = &accounts.seller_trade_state;
    let free_seller_trade_state = &accounts.free_seller_trade_state;
    let auction_house = &accounts.auction_house;
    let auction_house_fee_account = &accounts.auction_house_fee_account;
    let token_program = &accounts.token_program;
    let system_program = &accounts.system_program;
    let program_as_signer = &accounts.program_as_signer;
    let rent = &accounts.rent;

    // 1. The wallet being a signer is the only condition in which an NFT can sell at a price of 0.
    //    If the user does list at 0 then auction house can change the sale price if the 'can_change_sale_price' option is true.
    // 2. If the trade is not priced at 0, the wallet holder has to be a signer since auction house cannot sign if listing over 0.
    // 3. There must be one and only one signer; there can never be zero signers or two signers.
    // 4. Auction house should be the signer for changing the price instead of user wallet for cases when seller lists at 0.
    if !wallet.to_account_info().is_signer
        && (buyer_price == 0
            || free_seller_trade_state.data_is_empty()
            || !authority.to_account_info().is_signer
            || !auction_house.can_change_sale_price)
    {
        return Err(AuctionHouseError::SaleRequiresSigner.into());
    }

    let auction_house_key = auction_house.key();

    let seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        FEE_PAYER.as_bytes(),
        &[auction_house.fee_payer_bump],
    ];

    let (fee_payer, fee_seeds) = get_fee_payer(
        authority,
        auction_house,
        wallet.to_account_info(),
        auction_house_fee_account.to_account_info(),
        &seeds,
    )?;
    assert_is_ata(
        &token_account.to_account_info(),
        &wallet.key(),
        &token_account.mint,
    )?;

    assert_metadata_valid(metadata, token_account)?;

    if token_size > token_account.amount {
        return Err(AuctionHouseError::InvalidTokenAmount.into());
    }

    if wallet.is_signer {
        invoke(
            &approve(
                &token_program.key(),
                &token_account.key(),
                &program_as_signer.key(),
                &wallet.key(),
                &[],
                token_size,
            )
            .unwrap(),
            &[
                token_program.to_account_info(),
                token_account.to_account_info(),
                program_as_signer.to_account_info(),
                wallet.to_account_info(),
            ],
        )?;
    }

    let ts_info = seller_trade_state.to_account_info();
    if ts_info.data_is_empty() {
        let token_account_key = token_account.key();
        let wallet_key = wallet.key();
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
            *program_id,
            &ts_info,
            &rent.to_account_info(),
            system_program,
            &fee_payer,
            TRADE_STATE_SIZE,
            fee_seeds,
            &ts_seeds,
        )?;
    }

    let data = &mut ts_info.data.borrow_mut();
    data[0] = trade_state_bump;

    Ok(())
}
