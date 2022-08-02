use anchor_lang::prelude::*;

use crate::{constants::*, errors::*, utils::*, *};

pub mod auctioneer_withdraw;
pub mod withdraw;

pub use auctioneer_withdraw::*;
pub use withdraw::*;

#[allow(clippy::needless_lifetimes)]
fn withdraw_logic<'info>(
    accounts: &mut Withdraw<'info>,
    escrow_payment_bump: u8,
    amount: u64,
) -> Result<()> {
    let wallet = &accounts.wallet;
    let receipt_account = &accounts.receipt_account;
    let escrow_payment_account = &accounts.escrow_payment_account;
    let authority = &accounts.authority;
    let auction_house = &accounts.auction_house;
    let auction_house_fee_account = &accounts.auction_house_fee_account;
    let treasury_mint = &accounts.treasury_mint;
    let system_program = &accounts.system_program;
    let token_program = &accounts.token_program;
    let ata_program = &accounts.ata_program;
    let rent = &accounts.rent;

    let auction_house_key = auction_house.key();
    let seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        FEE_PAYER.as_bytes(),
        &[auction_house.fee_payer_bump],
    ];

    let ah_seeds = [
        PREFIX.as_bytes(),
        auction_house.creator.as_ref(),
        auction_house.treasury_mint.as_ref(),
        &[auction_house.bump],
    ];

    let auction_house_key = auction_house.key();
    let wallet_key = wallet.key();

    if !wallet.to_account_info().is_signer && !authority.to_account_info().is_signer {
        return Err(AuctionHouseError::NoValidSignerPresent.into());
    }

    let escrow_signer_seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        wallet_key.as_ref(),
        &[escrow_payment_bump],
    ];

    let (fee_payer, fee_seeds) = get_fee_payer(
        authority,
        auction_house,
        wallet.to_account_info(),
        auction_house_fee_account.to_account_info(),
        &seeds,
    )?;

    let is_native = treasury_mint.key() == spl_token::native_mint::id();

    if !is_native {
        if receipt_account.data_is_empty() {
            make_ata(
                receipt_account.to_account_info(),
                wallet.to_account_info(),
                treasury_mint.to_account_info(),
                fee_payer.to_account_info(),
                ata_program.to_account_info(),
                token_program.to_account_info(),
                system_program.to_account_info(),
                rent.to_account_info(),
                fee_seeds,
            )?;
        }

        let rec_acct = assert_is_ata(
            &receipt_account.to_account_info(),
            &wallet.key(),
            &treasury_mint.key(),
        )?;

        // make sure you cant get rugged
        if rec_acct.delegate.is_some() {
            return Err(AuctionHouseError::BuyerATACannotHaveDelegate.into());
        }

        assert_is_ata(receipt_account, &wallet.key(), &treasury_mint.key())?;
        invoke_signed(
            &spl_token::instruction::transfer(
                token_program.key,
                &escrow_payment_account.key(),
                &receipt_account.key(),
                &auction_house.key(),
                &[],
                amount,
            )?,
            &[
                escrow_payment_account.to_account_info(),
                receipt_account.to_account_info(),
                token_program.to_account_info(),
                auction_house.to_account_info(),
            ],
            &[&ah_seeds],
        )?;
    } else {
        assert_keys_equal(receipt_account.key(), wallet.key())?;
        let rent_shortfall = verify_withdrawal(escrow_payment_account.to_account_info(), amount)?;
        let checked_amount = amount
            .checked_sub(rent_shortfall)
            .ok_or(AuctionHouseError::InsufficientFunds)?;

        invoke_signed(
            &system_instruction::transfer(
                &escrow_payment_account.key(),
                &receipt_account.key(),
                checked_amount,
            ),
            &[
                escrow_payment_account.to_account_info(),
                receipt_account.to_account_info(),
                system_program.to_account_info(),
            ],
            &[&escrow_signer_seeds],
        )?;
    }

    Ok(())
}
